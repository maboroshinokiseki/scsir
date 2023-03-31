use std::time::Duration;

use scsir::{self, Scsi};

fn main() -> scsir::Result<()> {
    let mut scsi = scsir::Scsi::new("/dev/sdX")?;
    scsi.set_timeout(Duration::from_secs(3));

    println!("{:x?}", read_lbas(&scsi, 0, 1)?);

    Ok(())
}

fn read_lbas(interface: &Scsi, lba_offset: u64, lba_count: u64) -> scsir::Result<Vec<u8>> {
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
    // for memory usage and read efficiency
    let optimal_transfer_length = u32::min(
        optimal_transfer_length,
        32768 / optimal_transfer_length_granularity as u32
            * optimal_transfer_length_granularity as u32,
    );

    // it's actually last_lba_offset + 1
    let last_lba_offset = lba_offset + lba_count;
    let mut lba_offset = lba_offset;

    let mut bytes = vec![];

    while lba_offset < last_lba_offset {
        let lba_count = u64::min(last_lba_offset - lba_offset, optimal_transfer_length as u64);
        let data = interface
            .read()
            .logical_block_address(lba_offset)
            .logical_block_size(cap.logical_block_length_in_bytes)
            .transfer_length(lba_count as u32)
            .issue_16()?;

        bytes.extend_from_slice(&data);

        lba_offset += lba_count;
    }

    Ok(bytes)
}
