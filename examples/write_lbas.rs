use std::time::Duration;

use scsir::{self, Scsi};

fn main() -> scsir::Result<()> {
    let mut scsi = scsir::Scsi::new("/dev/sdX")?;
    scsi.set_timeout(Duration::from_secs(3));

    // dangerous!!!
    // write_lbas(&scsi, 0, &[0x0F; 512])?;

    Ok(())
}

#[allow(dead_code)]
fn write_lbas(interface: &Scsi, lba_offset: u64, bytes: &[u8]) -> scsir::Result<()> {
    let cap = interface.read_capacity().issue_16()?;
    let block_limit = scsir::shortcut::inquiry::block_limits(&mut interface.inquiry())?;

    let optimal_transfer_length = if block_limit.optimal_transfer_length == 0 {
        32768
    } else {
        block_limit.optimal_transfer_length
    };
    let optimal_transfer_length_granularity =
        if block_limit.optimal_transfer_length_granularity == 0 {
            u32::min(optimal_transfer_length, 32768) as u16
        } else {
            block_limit.optimal_transfer_length_granularity
        };
    // for write efficiency
    let optimal_transfer_length = optimal_transfer_length
        / optimal_transfer_length_granularity as u32
        * optimal_transfer_length_granularity as u32;

    let mut lba_offset = lba_offset;

    for bytes in
        bytes.chunks((optimal_transfer_length * cap.logical_block_length_in_bytes) as usize)
    {
        interface
            .write()
            .logical_block_address(lba_offset)
            .logical_block_size(cap.logical_block_length_in_bytes)
            .parameter(bytes)
            .issue_16()?;

        lba_offset += optimal_transfer_length as u64;
    }

    Ok(())
}
