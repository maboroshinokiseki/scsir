use std::io;

use crate::{command::sense::SenseData, error};

#[cfg(target_os = "linux")]
use crate::os::linux::{DriverStatus, HostStatus};

#[derive(Debug)]
pub struct ResultData<'a, D> {
    pub(crate) ioctl_result: i32,
    pub(crate) transfered_data_length: usize,
    pub(crate) data: &'a mut D,
    pub(crate) transfered_sense_length: usize,
    pub(crate) sense_buffer: &'a SenseData,
    pub(crate) status: Status,
    #[cfg(target_os = "linux")]
    pub(crate) host_status: HostStatus,
    #[cfg(target_os = "linux")]
    pub(crate) driver_status: DriverStatus,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum Status {
    Good,
    CheckCondition,
    ConditionMet,
    Busy,
    ReservationConflict,
    TaskSetFull,
    AcaActive,
    TaskAborted,
    Unknown(u8),
}

impl<D> ResultData<'_, D> {
    pub fn check_common_error(&self) -> crate::Result<()> {
        let mut result = String::new();

        #[cfg(target_os = "linux")]
        {
            if !matches!(self.host_status, crate::os::linux::HostStatus::Ok) {
                result.push_str(&format!("host status: {:?}. ", self.host_status));
            }

            if !self.driver_status.is_empty() {
                result.push_str(&format!("driver status: {:?}. ", self.driver_status));
            }
        }

        if !matches!(self.status, Status::Good) {
            result.push_str(&format!("Status: {:?}. ", self.status));
        }

        if self.transfered_sense_length != 0 {
            result.push_str(&format!("Sense data: {:02X?}", self.sense_buffer));
        }

        if !result.is_empty() {
            return Err(crate::Error::Other(result));
        }

        Ok(())
    }

    pub fn check_ioctl_error(&self) -> crate::Result<()> {
        match self.ioctl_result {
            0 => Ok(()),
            _ => Err(error::Error::IO(io::Error::last_os_error())),
        }
    }

    pub fn ioctl_result(&self) -> i32 {
        self.ioctl_result
    }

    pub fn transfered_data_length(&self) -> usize {
        self.transfered_data_length
    }

    pub fn data(&self) -> &D {
        self.data
    }

    pub fn transfered_sense_length(&self) -> usize {
        self.transfered_sense_length
    }

    pub fn sense_buffer(&self) -> &SenseData {
        self.sense_buffer
    }
}

impl From<u8> for Status {
    fn from(value: u8) -> Self {
        match value {
            0x00 => Self::Good,
            0x02 => Self::CheckCondition,
            0x04 => Self::ConditionMet,
            0x08 => Self::Busy,
            0x18 => Self::ReservationConflict,
            0x28 => Self::TaskSetFull,
            0x30 => Self::AcaActive,
            0x40 => Self::TaskAborted,
            _ => Self::Unknown(value),
        }
    }
}
