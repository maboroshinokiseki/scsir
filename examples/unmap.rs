use std::time::Duration;

use scsir::{self, Scsi};

fn main() -> scsir::Result<()> {
    let mut scsi = scsir::Scsi::new("/dev/sdX")?;
    scsi.set_timeout(Duration::from_secs(3));

    #[allow(unused_variables)]
    let cap = scsi.read_capacity().issue_16()?;

    let block_limit = scsir::shortcut::inquiry::block_limits(&mut scsi.inquiry())?;

    if block_limit.maximum_unmap_lba_count == 0 {
        return Err(scsir::Error::Other("Unmap Unsupported.".to_owned()));
    }

    // unmap entire disk, very dangerous!!!
    // unmap(&scsi,0,cap.returned_logical_block_address, block_limit.maximum_unmap_lba_count)?;

    Ok(())
}

#[allow(dead_code)]
fn unmap(
    interface: &Scsi,
    lba_offset: u64,
    lba_count: u64,
    max_unmap_lba_per_cycle: u32,
) -> scsir::Result<()> {
    // it's actually last_lba + 1
    let last_lba = lba_offset.saturating_add(lba_count);
    let mut lba_offset = lba_offset;
    while lba_offset < last_lba {
        let lba_count = u64::min(last_lba - lba_offset, max_unmap_lba_per_cycle as u64);
        interface
            .unmap()
            .parameter()
            .add_block_descriptor(lba_offset, lba_count as u32)
            .done()
            .unwrap()
            .issue()?;

        lba_offset += lba_count;
    }

    Ok(())
}
