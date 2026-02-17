#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

use crate::{
    command::bitfield_bound_check,
    data_wrapper::{AnyType, VecBufferWrapper},
    result_data::ResultData,
    Command, DataDirection, Scsi,
};

/// Determines the data flow direction between SAT layer and ATA device.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SatDirection {
    /// T_DIR = 0
    ToDevice,
    /// T_DIR = 1
    FromDevice,
}
impl SatDirection {
    pub fn to_data_direction(&self) -> DataDirection {
        match self {
            SatDirection::ToDevice => DataDirection::ToDevice,
            SatDirection::FromDevice => DataDirection::FromDevice,
        }
    }
}

/// Determines the protocol the SAT layer should use when talking to the ATA device.#
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum AtaProtocol {
    /// Device Management - ATA hardware reset
    HardwareReset = 0x00,
    /// Device Management - ATA software reset
    SoftwareReset = 0x01,
    /// Reserved
    Reserved02 = 0x02,
    /// Non-Data
    NonData = 0x03,
    /// PIO Data-In
    PioDataIn = 0x04,
    /// PIO Data-Out
    PioDataOut = 0x05,
    /// DMA
    Dma = 0x06,
    /// Reserved
    Reserved07 = 0x07,
    /// Execute Device Diagnostic
    ExecuteDeviceDiagnostic = 0x08,
    /// Non-data command - Device Reset
    DeviceReset = 0x09,
    /// UDMA Data In
    UdmaDataIn = 0x0A,
    /// UDMA Data Out
    UdmaDataOut = 0x0B,
    /// NCQ (see SATA 3.3)
    Ncq = 0x0C,
    /// Reserved
    Reserved0D = 0x0D,
    /// Reserved
    Reserved0E = 0x0E,
    /// Return Response Information
    ReturnResponseInformation = 0x0F,
}

#[derive(Clone, Debug)]
pub struct AtaPassThroughCommand<'a> {
    interface: &'a Scsi,
    dir: SatDirection,
    protocol: AtaProtocol,
    features: u16,
    lba: u64,
    count: Option<u16>,
    device: u8,
    command: u8,
    control: u8,
    data_buffer: Vec<u8>,
}

impl<'a> AtaPassThroughCommand<'a> {
    fn new(interface: &'a Scsi) -> Self {
        Self {
            interface,
            dir: SatDirection::ToDevice,
            protocol: AtaProtocol::PioDataOut,
            features: 0,
            lba: 0,
            count: None,
            device: 0,
            command: 0,
            control: 0,
            data_buffer: vec![],
        }
    }

    pub fn command(&mut self, dir: SatDirection, protocol: AtaProtocol, command: u8) -> &mut Self {
        self.dir = dir;
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

    pub fn lba(&mut self, lba: u64) -> &mut Self {
        self.lba = lba;
        self
    }

    pub fn count(&mut self, count: u16) -> &mut Self {
        self.count = Some(count);
        self
    }

    pub fn parameter(&mut self, value: &[u8]) -> &mut Self {
        self.data_buffer.clear();
        self.data_buffer.extend_from_slice(value);
        self
    }

    pub fn issue_12(&mut self) -> crate::Result<Option<Vec<u8>>> {
        bitfield_bound_check!(self.features, 8, "features")?;
        bitfield_bound_check!(self.lba, 24, "lba")?;
        let count = self.count.unwrap_or(self.data_buffer.len() as u16);
        assert!(count % 512 == 0, "buffer size has to be a multiple of 512");
        let sector_count = count / 512;
        bitfield_bound_check!(sector_count, 8, "count")?;
        self.data_buffer.resize(count as usize, 0);

        let lba = self.lba.to_le_bytes();

        let command_buffer = CommandBuffer12::new()
            .with_operation_code(OPERATION_CODE_12)
            .with_t_dir(self.dir as u8)
            .with_protocol(self.protocol as u8)
            // Tell SATL to take parameter length (in number of 512b-blocks) from count(0:7)
            .with_byte_block(1)
            .with_t_type(0)
            .with_t_length(0b10)
            //
            .with_features(self.features as u8)
            .with_count((count / 512) as u8)
            .with_lba_0(lba[0])
            .with_lba_1(lba[1])
            .with_lba_2(lba[2])
            .with_device(self.device)
            .with_command(self.command)
            .with_control(self.control);

        self.interface.issue(&ThisCommand {
            command_buffer,
            dir: self.dir.to_data_direction(),
            data_buffer: self.data_buffer.clone().into(),
        })
    }

    pub fn issue_16(&mut self) -> crate::Result<Option<Vec<u8>>> {
        bitfield_bound_check!(self.features, 16, "features")?;
        bitfield_bound_check!(self.lba, 24, "lba")?;
        let count = self.count.unwrap_or(self.data_buffer.len() as u16);
        assert!(count % 512 == 0, "buffer size has to be a multiple of 512");
        let sector_count = count / 512;
        bitfield_bound_check!(sector_count, 8, "count")?;
        self.data_buffer.resize(count as usize, 0);

        let features = self.features.to_le_bytes();
        let lba = self.lba.to_le_bytes();
        let count = (count / 512).to_le_bytes();

        let command_buffer = CommandBuffer16::new()
            .with_operation_code(OPERATION_CODE_16)
            .with_t_dir(self.dir as u8)
            .with_protocol(self.protocol as u8)
            // Tell SATL to take parameter length (in number of 512b-blocks) from count(0:7)
            .with_byte_block(1)
            .with_t_type(0)
            .with_t_length(0b10)
            //
            .with_features_low(features[0])
            .with_features_high(features[1])
            .with_count_low(count[0])
            .with_count_high(count[1])
            .with_lba_0(lba[0])
            .with_lba_1(lba[1])
            .with_lba_2(lba[2])
            .with_lba_3(lba[3])
            .with_lba_4(lba[4])
            .with_lba_5(lba[5])
            .with_device(self.device)
            .with_command(self.command)
            .with_control(self.control);

        self.interface.issue(&ThisCommand {
            command_buffer,
            dir: self.dir.to_data_direction(),
            data_buffer: self.data_buffer.clone().into(),
        })
    }
}

impl Scsi {
    pub fn ata_passthru(&self) -> AtaPassThroughCommand<'_> {
        AtaPassThroughCommand::new(self)
    }
}

const OPERATION_CODE_12: u8 = 0xA1;
const OPERATION_CODE_16: u8 = 0x85;

#[bitfield]
#[derive(Clone, Copy)]
struct CommandBuffer12 {
    operation_code: B8,
    obsolete_0: B3,
    protocol: B4,
    reserved_0: B1,
    off_line: B2,
    ck_cond: B1,
    t_type: B1,
    t_dir: B1,
    byte_block: B1,
    t_length: B2,
    features: B8,
    count: B8,
    lba_0: B8,
    lba_1: B8,
    lba_2: B8,
    device: B8,
    command: B8,
    reserved_1: B8,
    control: B8,
}

#[bitfield]
#[derive(Clone, Copy)]
struct CommandBuffer16 {
    operation_code: B8,
    obsolete_0: B3,
    protocol: B4,
    extend: B1,
    off_line: B2,
    ck_cond: B1,
    t_type: B1,
    t_dir: B1,
    byte_block: B1,
    t_length: B2,
    features_high: B8,
    features_low: B8,
    count_high: B8,
    count_low: B8,
    lba_3: B8,
    lba_0: B8,
    lba_4: B8,
    lba_1: B8,
    lba_5: B8,
    lba_2: B8,
    device: B8,
    command: B8,
    control: B8,
}

struct ThisCommand<C> {
    command_buffer: C,
    dir: DataDirection,
    data_buffer: VecBufferWrapper,
}

impl<C: Copy> Command for ThisCommand<C> {
    type CommandBuffer = C;
    type DataBuffer = AnyType;
    type DataBufferWrapper = VecBufferWrapper;
    type ReturnType = crate::Result<Option<Vec<u8>>>;

    fn direction(&self) -> DataDirection {
        self.dir
    }

    fn command(&self) -> Self::CommandBuffer {
        self.command_buffer
    }

    fn data(&self) -> Self::DataBufferWrapper {
        self.data_buffer.clone()
    }

    fn data_size(&self) -> u32 {
        self.data_buffer.len() as u32
    }

    fn process_result(&self, result: ResultData<Self::DataBufferWrapper>) -> Self::ReturnType {
        result.check_ioctl_error()?;
        result.check_common_error()?;

        match self.dir {
            DataDirection::ToDevice => Ok(None),
            DataDirection::FromDevice => Ok(Some(std::mem::take(result.data).0)),
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    const COMMAND_LENGTH_12: usize = 12;
    const COMMAND_LENGTH_16: usize = 16;

    #[test]
    fn layout_test() {
        assert_eq!(
            size_of::<CommandBuffer12>(),
            COMMAND_LENGTH_12,
            concat!("Size of: ", stringify!(CommandBuffer12))
        );

        assert_eq!(
            size_of::<CommandBuffer16>(),
            COMMAND_LENGTH_16,
            concat!("Size of: ", stringify!(CommandBuffer16))
        );
    }
}
