use windows::Win32::Storage::IscsiDisc::SCSI_PASS_THROUGH_DIRECT;

use crate::command::sense::MAX_SENSE_BUFFER_LENGTH;

#[repr(C)]
pub struct ScsiPassThroughDirectWrapper {
    pub scsi_pass_through: SCSI_PASS_THROUGH_DIRECT,
    pub sense: [u8; MAX_SENSE_BUFFER_LENGTH],
}

impl Default for ScsiPassThroughDirectWrapper {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}
