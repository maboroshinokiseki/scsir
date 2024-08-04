use crate::ResultData;

use super::{DriverStatus, HostStatus};

#[allow(dead_code)]
pub trait ResultDataExt {
    fn host_status(&self) -> &HostStatus;
    fn driver_status(&self) -> &DriverStatus;
}

impl<'a, D> ResultDataExt for ResultData<'a, D> {
    fn host_status(&self) -> &HostStatus {
        &self.host_status
    }

    fn driver_status(&self) -> &DriverStatus {
        &self.driver_status
    }
}
