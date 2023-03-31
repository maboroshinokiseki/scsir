use std::time::Duration;

use scsir::{self, Scsi};

fn main() -> scsir::Result<()> {
    let mut scsi = scsir::Scsi::new("/dev/sdX")?;
    scsi.set_timeout(Duration::from_secs(3));

    println!("{:x?}", read_bytes(&scsi, 0, 512)?);

    Ok(())
}

fn read_bytes(interface: &Scsi, byte_offset: u64, byte_count: u64) -> scsir::Result<Vec<u8>> {
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
    // it's actually last_byte_offset + 1
    let last_byte_offset = byte_offset.saturating_add(byte_count);
    // it's actually last_lba_offset + 1
    let last_lba_offset = if last_byte_offset % cap.logical_block_length_in_bytes as u64 == 0 {
        last_byte_offset / cap.logical_block_length_in_bytes as u64
    } else {
        last_byte_offset / cap.logical_block_length_in_bytes as u64 + 1
    };
    let mut byte_count_left = byte_count;
    let mut lba_offset = byte_offset / cap.logical_block_length_in_bytes as u64;
    let mut skip_byte_count = byte_offset % cap.logical_block_length_in_bytes as u64;

    let mut bytes = vec![];

    while lba_offset < last_lba_offset {
        let lba_count = u64::min(last_lba_offset - lba_offset, optimal_transfer_length as u64);
        let data = interface
            .read()
            .logical_block_address(lba_offset)
            .logical_block_size(cap.logical_block_length_in_bytes)
            .transfer_length(lba_count as u32)
            .issue_16()?;

        let mut data = &data[..];

        if skip_byte_count != 0 {
            data = &data[skip_byte_count as usize..];
            skip_byte_count = 0;
        }

        data = &data[..usize::min(
            data.len(),
            u64::min(byte_count_left, usize::MAX as u64) as usize,
        )];
        bytes.extend_from_slice(data);

        lba_offset += lba_count;
        byte_count_left -= data.len() as u64;
    }

    Ok(bytes)
}
