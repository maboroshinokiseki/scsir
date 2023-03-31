# SCSIR
A simple library for issuing SCSI commands.

# Example
```rust
fn main() -> scsir::Result<()> {
    // Open a scsi device
    let scsi = scsir::Scsi::new("/dev/sdX")?;

    // Issue a simple command
    scsi.test_unit_ready().issue()?;

    // Issue a command with parameters(fields)
    scsi.read()
        .logical_block_address(0)
        .logical_block_size(512)
        .transfer_length(1)
        .issue_16()?;

    // Issue a command with data out buffer parameters
    scsi.verify()
        .byte_check(0b01)
        .logical_block_address(0)
        .parameter(&[0; 512])
        .issue_16()?;
}
```
