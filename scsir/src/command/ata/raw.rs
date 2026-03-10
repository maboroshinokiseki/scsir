use std::marker::PhantomData;

use crate::{
    command::{
        ata::{
            AtaProtocol, FromDevice, NoData, SatCommand, SatCommandBuffer12, SatCommandBuffer16,
            SatDirection, SatResult, ScsiSat, ToDevice, OPERATION_CODE_12, OPERATION_CODE_16,
        },
        bitfield_bound_check,
    },
    Scsi,
};

const ATA_USING_LBA: u8 = (1 << 6);

macro_rules! define_byte_setter_fn {
    ($name:ident, $field:ident : $ty:ty, $index:expr) => {
        #[inline]
        pub fn $name(&mut self, b: u8) -> &mut Self {
            const SHIFT: usize = $index * 8;
            // Mask for one byte at the correct position
            let mask: $ty = !(((0xFF as $ty) << SHIFT) as $ty);
            // Clear target byte, then insert new one
            self.$field = (self.$field & mask) | (((b as $ty) << SHIFT) as $ty);
            self
        }
    };
}

#[derive(Clone, Debug)]
#[allow(private_bounds)]
pub struct RawSatCommand<'a, D: SatDirection> {
    interface: &'a Scsi,
    protocol: AtaProtocol,
    ck_cond: bool,

    features: u16,
    lba: u64,
    device: u8,
    command: u8,
    control: u8,
    data_buffer: Vec<u8>,

    _direction: PhantomData<D>,
}

#[allow(private_bounds)]
impl<'a, D: SatDirection> RawSatCommand<'a, D> {
    fn new(interface: &'a Scsi) -> Self {
        Self {
            interface,
            protocol: AtaProtocol::PioDataOut,
            ck_cond: false,

            features: 0,
            lba: 0,
            device: ATA_USING_LBA,
            command: 0,
            control: 0,
            data_buffer: vec![],

            _direction: Default::default(),
        }
    }

    pub fn ck_cond(&mut self, ck_cond: bool) -> &mut Self {
        self.ck_cond = ck_cond;
        self
    }

    pub fn command(&mut self, protocol: AtaProtocol, command: u8) -> &mut Self {
        self.protocol = protocol;
        self.command = command;
        self
    }

    pub fn device(&mut self, device: u8) -> &mut Self {
        self.device = device;
        self
    }

    pub fn control(&mut self, control: u8) -> &mut Self {
        self.control = control;
        self
    }

    pub fn features(&mut self, features: u16) -> &mut Self {
        self.features = features;
        self
    }
    define_byte_setter_fn!(feature_0_7, features: u16, 0);
    define_byte_setter_fn!(feature_8_15, features: u16, 1);

    pub fn lba(&mut self, lba: u64) -> &mut Self {
        self.lba = lba;
        self
    }
    define_byte_setter_fn!(lba_0_7, lba: u64, 0);
    define_byte_setter_fn!(lba_8_15, lba: u64, 1);
    define_byte_setter_fn!(lba_16_23, lba: u64, 2);
    define_byte_setter_fn!(lba_24_31, lba: u64, 3);
    define_byte_setter_fn!(lba_32_39, lba: u64, 4);
    define_byte_setter_fn!(lba_40_47, lba: u64, 5);
    define_byte_setter_fn!(lba_48_55, lba: u64, 6);
    define_byte_setter_fn!(lba_56_63, lba: u64, 7);

    pub fn count(&mut self, count: u16) -> &mut Self {
        self.data_buffer.resize(count as usize, 0);
        self
    }

    pub fn parameter(&mut self, value: &[u8]) -> &mut Self {
        self.data_buffer.clear();
        self.data_buffer.extend_from_slice(value);
        self
    }

    // ######################################################################

    fn check_and_calculate_count(
        &self,
        features_bits: u32,
        sector_count_bits: u32,
        lba_bits: u32,
    ) -> crate::Result<u16> {
        bitfield_bound_check!(self.features, features_bits, "features")?;
        bitfield_bound_check!(self.lba, lba_bits, "lba")?;
        let count = self.data_buffer.len();
        if count % 512 != 0 {
            let msg = format!("Data buffer size (count) must be: {count}");
            return Err(crate::Error::BadArgument(msg));
        }
        let sector_count = (count / 512) as u16;
        bitfield_bound_check!(sector_count, sector_count_bits, "count")?;
        Ok(sector_count)
    }

    pub(crate) fn build_cmd_12(&self) -> crate::Result<SatCommandBuffer12> {
        let sector_count = self.check_and_calculate_count(8, 8, 24)?;
        let lba = self.lba.to_le_bytes();

        Ok(SatCommandBuffer12::new()
            .with_operation_code(OPERATION_CODE_12)
            .with_t_dir(D::T_DIR)
            .with_protocol(self.protocol as u8)
            .with_ck_cond(self.ck_cond as u8)
            .with_t_type(0)
            // Tell SATL to take parameter length (in number of 512b-blocks) from count(0:7)
            .with_byte_block(if D::HAS_DATA { 1 } else { 0 })
            .with_t_length(if D::HAS_DATA { 0b10 } else { 0 })
            //
            .with_features(self.features as u8)
            .with_count(sector_count as u8)
            .with_lba_0(lba[0])
            .with_lba_1(lba[1])
            .with_lba_2(lba[2])
            .with_device(self.device)
            .with_command(self.command)
            .with_control(self.control))
    }

    pub(crate) fn build_cmd_16(&self) -> crate::Result<SatCommandBuffer16> {
        let sector_count = self.check_and_calculate_count(16, 16, 48)?.to_le_bytes();
        let features = self.features.to_le_bytes();
        let lba = self.lba.to_le_bytes();

        Ok(SatCommandBuffer16::new()
            .with_operation_code(OPERATION_CODE_16)
            .with_t_dir(D::T_DIR)
            .with_protocol(self.protocol as u8)
            .with_ck_cond(self.ck_cond as u8)
            .with_t_type(0)
            // Tell SATL to take parameter length (in number of 512b-blocks) from count(0:7)
            .with_byte_block(if D::HAS_DATA { 1 } else { 0 })
            .with_t_length(if D::HAS_DATA { 0b10 } else { 0 })
            //
            .with_features_low(features[0])
            .with_features_high(features[1])
            .with_count_low(sector_count[0])
            .with_count_high(sector_count[1])
            .with_lba_0(lba[0])
            .with_lba_1(lba[1])
            .with_lba_2(lba[2])
            .with_lba_3(lba[3])
            .with_lba_4(lba[4])
            .with_lba_5(lba[5])
            .with_device(self.device)
            .with_command(self.command)
            .with_control(self.control))
    }
}

impl<'a> RawSatCommand<'a, ToDevice> {
    pub fn issue_12(&mut self) -> SatResult<()> {
        self.interface.issue(&SatCommand::<_, ToDevice> {
            command_buffer: self.build_cmd_12()?,
            data_buffer: self.data_buffer.clone().into(),
            ck_cond: self.ck_cond,
            _direction: Default::default(),
        })
    }

    pub fn issue_16(&mut self) -> SatResult<()> {
        self.interface.issue(&SatCommand::<_, ToDevice> {
            command_buffer: self.build_cmd_16()?,
            data_buffer: self.data_buffer.clone().into(),
            ck_cond: self.ck_cond,
            _direction: Default::default(),
        })
    }
}

impl<'a> RawSatCommand<'a, FromDevice> {
    pub fn issue_12(&mut self) -> SatResult<Vec<u8>> {
        self.interface.issue(&SatCommand::<_, FromDevice> {
            command_buffer: self.build_cmd_12()?,
            data_buffer: self.data_buffer.clone().into(),
            ck_cond: self.ck_cond,
            _direction: Default::default(),
        })
    }

    pub fn issue_16(&mut self) -> SatResult<Vec<u8>> {
        self.interface.issue(&SatCommand::<_, FromDevice> {
            command_buffer: self.build_cmd_16()?,
            data_buffer: self.data_buffer.clone().into(),
            ck_cond: self.ck_cond,
            _direction: Default::default(),
        })
    }
}

impl<'a> RawSatCommand<'a, NoData> {
    pub fn issue_12(&mut self) -> SatResult<()> {
        self.interface.issue(&SatCommand::<_, NoData> {
            command_buffer: self.build_cmd_12()?,
            data_buffer: self.data_buffer.clone().into(),
            ck_cond: self.ck_cond,
            _direction: Default::default(),
        })
    }

    pub fn issue_16(&mut self) -> SatResult<()> {
        self.interface.issue(&SatCommand::<_, NoData> {
            command_buffer: self.build_cmd_16()?,
            data_buffer: self.data_buffer.clone().into(),
            ck_cond: self.ck_cond,
            _direction: Default::default(),
        })
    }
}

impl ScsiSat<'_> {
    pub fn raw_read(&self) -> RawSatCommand<'_, FromDevice> {
        RawSatCommand::new(self.interface)
    }
    pub fn raw_write(&self) -> RawSatCommand<'_, ToDevice> {
        RawSatCommand::new(self.interface)
    }
    pub fn raw_nodata(&self) -> RawSatCommand<'_, NoData> {
        RawSatCommand::new(self.interface)
    }
}
