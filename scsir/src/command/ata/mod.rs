#![allow(dead_code)]

use std::marker::PhantomData;

use crate::{
    command::sense::SenseData,
    data_wrapper::{AnyType, VecBufferWrapper},
    result_data::{ResultData, Status},
    Command, DataDirection, Scsi,
};
use modular_bitfield_msb::prelude::*;

pub mod identify;
pub mod raw;
pub mod smart;

#[derive(Clone, Debug)]
pub struct SatResultData<T> {
    pub data: T,
    pub sense: SenseData,
}
impl<T> SatResultData<T> {
    pub fn map<O>(self, map_fn: impl FnOnce(T) -> O) -> SatResultData<O> {
        SatResultData {
            data: map_fn(self.data),
            sense: self.sense,
        }
    }
}

pub type SatResult<T> = crate::Result<SatResultData<T>>;

/// Determines the data flow direction between SAT layer and ATA device.
pub(crate) trait SatDirection {
    const T_DIR: u8;
    const HAS_DATA: bool;
    const DATA_DIRECTION: DataDirection;
    type ReturnType;

    fn process_result(result: ResultData<VecBufferWrapper>) -> crate::Result<Self::ReturnType>;
}

#[derive(Copy, Clone, Debug)]
pub struct ToDevice;
impl SatDirection for ToDevice {
    const T_DIR: u8 = 0;
    const HAS_DATA: bool = true;
    const DATA_DIRECTION: DataDirection = DataDirection::ToDevice;
    type ReturnType = ();

    fn process_result(_: ResultData<VecBufferWrapper>) -> crate::Result<Self::ReturnType> {
        Ok(())
    }
}

#[derive(Copy, Clone, Debug)]
pub struct FromDevice;
impl SatDirection for FromDevice {
    const T_DIR: u8 = 1;
    const HAS_DATA: bool = true;
    const DATA_DIRECTION: DataDirection = DataDirection::FromDevice;
    type ReturnType = Vec<u8>;

    fn process_result(result: ResultData<VecBufferWrapper>) -> crate::Result<Self::ReturnType> {
        Ok(std::mem::take(result.data).0)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct NoData;
impl SatDirection for NoData {
    const T_DIR: u8 = 0;
    const HAS_DATA: bool = false;
    const DATA_DIRECTION: DataDirection = DataDirection::None;
    type ReturnType = ();

    fn process_result(_: ResultData<VecBufferWrapper>) -> crate::Result<Self::ReturnType> {
        Ok(())
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

impl Scsi {
    pub fn sat(&self) -> ScsiSat<'_> {
        ScsiSat { interface: self }
    }
}

#[derive(Copy, Clone)]
pub struct ScsiSat<'a> {
    interface: &'a Scsi,
}

const OPERATION_CODE_12: u8 = 0xA1;
const OPERATION_CODE_16: u8 = 0x85;

#[bitfield]
#[derive(Clone, Copy)]
pub(crate) struct SatCommandBuffer12 {
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
pub(crate) struct SatCommandBuffer16 {
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

pub(crate) struct SatCommand<C, D: SatDirection> {
    command_buffer: C,
    data_buffer: VecBufferWrapper,
    // Whether the request explicitly requested ck_cond
    ck_cond: bool,
    _direction: PhantomData<D>,
}

impl<C: Copy, D: SatDirection> SatCommand<C, D> {
    /// Check for common error conditions
    /// Don't regard SENSE as error, if it was explicitly requested by the SAT command (ck_cond)
    fn check_common_error(&self, result: &ResultData<VecBufferWrapper>) -> crate::Result<()> {
        let mut err = String::new();

        #[cfg(target_os = "linux")]
        {
            use crate::os::linux::DriverStatus;

            if !matches!(result.host_status, crate::os::linux::HostStatus::Ok) {
                err.push_str(&format!("host status: {:?}. ", result.host_status));
            }

            match (self.ck_cond, result.driver_status) {
                (_, _) if result.driver_status.is_empty() => {}
                (true, DriverStatus::SENSE) => {}
                _ => err.push_str(&format!("driver status: {:?}. ", result.driver_status)),
            }
        }

        match result.status {
            Status::Good => {}
            Status::CheckCondition if self.ck_cond => {}
            _ => err.push_str(&format!("Status: {:?}. ", result.status)),
        }

        if result.transfered_sense_length != 0 && !self.ck_cond {
            err.push_str(&format!("Sense data: {:02X?}", result.sense_buffer));
        }

        if !err.is_empty() {
            return Err(crate::Error::Other(err));
        }

        Ok(())
    }
}

impl<C: Copy, D: SatDirection> Command for SatCommand<C, D> {
    type CommandBuffer = C;
    type DataBuffer = AnyType;
    type DataBufferWrapper = VecBufferWrapper;
    type ReturnType = SatResult<D::ReturnType>;

    fn direction(&self) -> DataDirection {
        D::DATA_DIRECTION
    }

    fn command(&self) -> Self::CommandBuffer {
        self.command_buffer
    }

    fn data(&self) -> Self::DataBufferWrapper {
        match D::HAS_DATA {
            true => self.data_buffer.clone(),
            false => VecBufferWrapper::default(),
        }
    }

    fn data_size(&self) -> u32 {
        match D::HAS_DATA {
            true => self.data_buffer.len() as u32,
            false => 0,
        }
    }

    fn process_result(&self, result: ResultData<Self::DataBufferWrapper>) -> Self::ReturnType {
        result.check_ioctl_error()?;
        self.check_common_error(&result)?;
        Ok(SatResultData {
            sense: result.sense_buffer().clone(),
            data: D::process_result(result)?,
        })
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
            size_of::<SatCommandBuffer12>(),
            COMMAND_LENGTH_12,
            concat!("Size of: ", stringify!(CommandBuffer12))
        );

        assert_eq!(
            size_of::<SatCommandBuffer16>(),
            COMMAND_LENGTH_16,
            concat!("Size of: ", stringify!(CommandBuffer16))
        );
    }
}
