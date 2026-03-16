use crate::command::ata::{raw::RawSatCommand, AtaProtocol, NoData, SatResult, ScsiSat};

const OPCODE_FLUSH_CACHE: u8 = 0xE7;

#[derive(Clone, Debug)]
pub struct FlushCacheSatCommand<'a> {
    raw: RawSatCommand<'a, NoData>,
}

impl<'a> FlushCacheSatCommand<'a> {
    fn new(sat: &'a ScsiSat<'a>) -> Self {
        let mut raw = sat.raw_nodata();
        raw.command(AtaProtocol::NonData, OPCODE_FLUSH_CACHE);
        Self { raw }
    }

    pub fn device(&mut self, device: u8) -> &mut Self {
        self.raw.device(device);
        self
    }

    // ######################################################################

    pub fn issue_12(&mut self) -> SatResult<()> {
        self.raw.issue_12()
    }

    pub fn issue_16(&mut self) -> SatResult<()> {
        self.raw.issue_16()
    }
}

impl ScsiSat<'_> {
    /// Command: FLUSH CACHE (0xE7)
    pub fn flush_cache(&self) -> FlushCacheSatCommand<'_> {
        FlushCacheSatCommand::new(self)
    }
}
