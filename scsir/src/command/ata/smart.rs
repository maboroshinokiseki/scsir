use crate::command::ata::{raw::RawSatCommand, AtaProtocol, FromDevice, SatResult, ScsiSat};

const OPCODE_SMART: u8 = 0xB0;
const SUBCMD_READ_DATA: u8 = 0xD0;
const SUBCMD_READ_LOG: u8 = 0xD5;

#[derive(Clone, Debug)]
pub struct SmartSatCommand<'a> {
    raw: RawSatCommand<'a, FromDevice>,
}

impl<'a> SmartSatCommand<'a> {
    fn new(sat: &'a ScsiSat<'a>, subcommand: u8, lba_7_0: u8) -> Self {
        let mut raw = sat.raw_read();
        raw.command(AtaProtocol::PioDataIn, OPCODE_SMART)
            .features(subcommand as u16)
            .lba_16_23(0xC2)
            .lba_8_15(0x4F)
            .lba_0_7(lba_7_0)
            .count(512);
        Self { raw }
    }

    pub fn device(&mut self, device: u8) -> &mut Self {
        self.raw.device(device);
        self
    }

    pub fn timeout(&mut self, timeout: std::time::Duration) -> &mut Self {
        self.raw.timeout(timeout);
        self
    }

    // ######################################################################

    pub fn issue_12(&mut self) -> SatResult<Vec<u8>> {
        self.raw.issue_12()
    }

    pub fn issue_16(&mut self) -> SatResult<Vec<u8>> {
        self.raw.issue_16()
    }
}

impl ScsiSat<'_> {
    /// Command: SMART READ DATA (0xD0)
    pub fn smart_read_data(&self) -> SmartSatCommand<'_> {
        SmartSatCommand::new(self, SUBCMD_READ_DATA, 0x00)
    }
    /// Command: SMART READ LOG (0xD5)
    pub fn smart_read_log(&self, addr: u8) -> SmartSatCommand<'_> {
        SmartSatCommand::new(self, SUBCMD_READ_LOG, addr)
    }
}
