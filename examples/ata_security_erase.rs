use std::time::Duration;

use scsir::command::ata::{
    identify::SecurityState,
    security::{EraseMode, SecurityMode, SecurityPassword},
};

fn main() -> scsir::Result<()> {
    let mut scsi = scsir::Scsi::new("/dev/sdX")?;

    let identify = scsi.sat().identify().issue_16().unwrap().data;
    if !identify.security_supported() || !identify.cap_feature_set_security() {
        panic!("This disk doesn't support the SECURITY feature-set");
    }
    if identify.security_state() != SecurityState::SEC1 {
        panic!("Disk currently in unsupported security state - security probably already enabled");
    }

    // make sure the disk controller flushes all outstanding IO ops
    if identify.enabled_cmd_flush_cache_ext() {
        scsi.sat().flush_cache_ext().issue_16().unwrap();
    } else if identify.enabled_cmd_flush_cache() {
        scsi.sat().flush_cache().issue_16().unwrap();
    }

    // disk controllers like to lie a lot - so give it some time
    std::thread::sleep(Duration::from_millis(5000));

    // enable SECURITY by setting user password
    scsi.sat()
        .security_set_password()
        .mode(SecurityMode::High)
        .password(SecurityPassword::User, "scsir".as_bytes())
        .issue_16()
        .unwrap();

    let identify = scsi.sat().identify().issue_16().unwrap().data;
    if identify.security_state() != SecurityState::SEC5 {
        panic!("Engaging drive SECURITY failed");
    }

    scsi.sat().security_erase_prepare().issue_16().unwrap();

    // temporarily overwrite command timeout for device with the time required for an
    // enhanced security erase (estimated by the drive's IDENTIFY data itself).
    let prev_timeout = scsi.timeout();
    scsi.set_timeout(identify.time_required_for_enhanced_erase() + Duration::from_mins(30));
    // perform actual secure erase
    scsi.sat()
        .security_erase_unit()
        .mode(EraseMode::Enhanced)
        .password(SecurityPassword::User, "scsir".as_bytes())
        .issue_16()
        .unwrap();
    scsi.set_timeout(prev_timeout);

    Ok(())
}
