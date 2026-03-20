use std::time::Duration;

use crate::command::ata::{raw::RawSatCommand, AtaProtocol, FromDevice, SatResult, ScsiSat};

const OPCODE_IDENTIFY_DEVICE: u8 = 0xEC;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SecurityState {
    SEC1,
    SEC2,
    SEC4,
    SEC5,
    SEC6,
    Unknown,
}

#[derive(Clone, Debug)]
pub struct IdentifySatCommand<'a> {
    raw: RawSatCommand<'a, FromDevice>,
}

impl<'a> IdentifySatCommand<'a> {
    fn new(sat: &'a ScsiSat<'a>) -> Self {
        let mut raw = sat.raw_read();

        raw.command(AtaProtocol::PioDataIn, OPCODE_IDENTIFY_DEVICE)
            .features(0)
            .lba_16_23(0)
            .lba_8_15(0)
            .lba_0_7(0)
            .count(512);

        Self { raw }
    }

    pub fn device(&mut self, device: u8) -> &mut Self {
        self.raw.device(device);
        self
    }

    pub fn timeout(&mut self, timeout: Duration) -> &mut Self {
        self.raw.timeout(timeout);
        self
    }

    pub fn issue_12(&mut self) -> SatResult<IdentifySatResponse> {
        Ok(self.raw.issue_12()?.map(IdentifySatResponse::new))
    }

    pub fn issue_16(&mut self) -> SatResult<IdentifySatResponse> {
        Ok(self.raw.issue_16()?.map(IdentifySatResponse::new))
    }
}

pub struct IdentifySatResponse {
    bfr: Vec<u8>,
}

macro_rules! define_bitflag_getter {
    ($name:ident, $word_idx:literal, $bit_idx:literal) => {
        pub fn $name(&self) -> bool {
            self.word($word_idx) & (1 << $bit_idx) != 0
        }
    };
}

impl IdentifySatResponse {
    pub fn new(bfr: Vec<u8>) -> Self {
        Self { bfr }
    }

    pub fn raw(&self) -> &Vec<u8> {
        &self.bfr
    }

    /// Helper: read a 16‑bit IDENTIFY word (little‑endian)
    fn word(&self, index: usize) -> u16 {
        let offset = index * 2;
        u16::from_le_bytes([self.bfr[offset], self.bfr[offset + 1]])
    }

    /// Decode ATA string fields (word‑swapped ASCII)
    fn ata_string(&self, start_word: usize, end_word: usize) -> String {
        let mut bytes = Vec::with_capacity((end_word - start_word) * 2);

        for w in start_word..end_word {
            let word = self.word(w);
            // ATA stores characters as: high byte first, then low byte
            bytes.push((word >> 8) as u8);
            bytes.push((word & 0xFF) as u8);
        }

        // Trim trailing spaces and convert to UTF‑8
        String::from_utf8_lossy(&bytes).trim().to_string()
    }

    // ------------------------------------------------------------------------------------------

    pub fn firmware_rev(&self) -> String {
        self.ata_string(23, 27)
    }

    pub fn model_nr(&self) -> String {
        self.ata_string(27, 47)
    }

    // word 49
    define_bitflag_getter!(cap_lba28, 49, 9);

    // word 59
    define_bitflag_getter!(cap_cmd_sanitize_antiifreeze_lock_ext, 59, 10);
    define_bitflag_getter!(cap_vers_sanitize_acs3, 59, 11);
    define_bitflag_getter!(cap_feature_set_sanitize, 59, 12);
    define_bitflag_getter!(cap_cmd_crypto_scramble_ext, 59, 13);
    define_bitflag_getter!(cap_cmd_overwrite_ext, 59, 14);
    define_bitflag_getter!(cap_cmd_block_erase_ext, 59, 15);

    // word 82
    define_bitflag_getter!(cap_feature_set_smart, 82, 0);
    define_bitflag_getter!(cap_feature_set_security, 82, 1);
    define_bitflag_getter!(cap_volatile_write_cache, 82, 5);
    define_bitflag_getter!(cap_read_look_ahead, 82, 6);
    define_bitflag_getter!(cap_cmd_write_buffer, 82, 12);
    define_bitflag_getter!(cap_cmd_read_buffer, 82, 13);
    define_bitflag_getter!(cap_cmd_nop, 82, 14);

    // word 83
    define_bitflag_getter!(cap_cmd_download_microcode, 83, 0);
    define_bitflag_getter!(cap_feature_set_apm, 83, 3);
    define_bitflag_getter!(cap_feature_set_puis, 83, 5);
    define_bitflag_getter!(cap_set_features_required_for_spinup, 83, 6);
    define_bitflag_getter!(cap_lba48, 83, 10);
    define_bitflag_getter!(cap_cmd_flush_cache, 83, 12);
    define_bitflag_getter!(cap_cmd_flush_cache_ext, 83, 13);

    // word 84
    define_bitflag_getter!(cap_smart_error_logging, 84, 0);
    define_bitflag_getter!(cap_smart_self_test, 84, 1);
    define_bitflag_getter!(cap_feature_set_streaming, 84, 4);
    define_bitflag_getter!(cap_feature_set_gpl, 84, 5);
    define_bitflag_getter!(cap_cmd_write_dma_fua_ext, 84, 6);
    define_bitflag_getter!(cap_cmd_idle_immediate_with_unload, 84, 13);

    // word 85
    define_bitflag_getter!(enabled_feature_set_smart, 85, 0);
    define_bitflag_getter!(enabled_feature_set_security, 85, 1);
    define_bitflag_getter!(enabled_volatile_write_cache, 85, 5);
    define_bitflag_getter!(enabled_read_look_ahead, 85, 6);
    define_bitflag_getter!(enabled_cmd_write_buffer, 85, 12);
    define_bitflag_getter!(enabled_cmd_read_buffer, 85, 13);
    define_bitflag_getter!(enabled_cmd_nop, 85, 14);

    // word 86
    define_bitflag_getter!(enabled_cmd_download_microcode, 86, 0);
    define_bitflag_getter!(enabled_feature_set_apm, 86, 3);
    define_bitflag_getter!(enabled_feature_set_puis, 86, 5);
    define_bitflag_getter!(enabled_set_features_required_for_spinup, 86, 6);
    define_bitflag_getter!(enabled_lba48, 86, 10);
    define_bitflag_getter!(enabled_cmd_flush_cache, 86, 12);
    define_bitflag_getter!(enabled_cmd_flush_cache_ext, 86, 13);

    // word 87
    define_bitflag_getter!(enabled_smart_error_logging, 87, 0);
    define_bitflag_getter!(enabled_smart_self_test, 87, 1);
    define_bitflag_getter!(enabled_feature_set_streaming, 87, 4);
    define_bitflag_getter!(enabled_feature_set_gpl, 87, 5);
    define_bitflag_getter!(enabled_cmd_write_dma_fua_ext, 87, 6);
    define_bitflag_getter!(enabled_cmd_idle_immediate_with_unload, 87, 13);

    // word 119
    define_bitflag_getter!(cap_feature_set_write_read_verify, 119, 1);
    define_bitflag_getter!(cap_cmd_write_uncorrectable_ext, 119, 2);
    define_bitflag_getter!(cap_cmd_read_log_dma_ext, 119, 3);
    define_bitflag_getter!(cap_feature_set_free_fall, 119, 5);
    define_bitflag_getter!(cap_feature_set_sense_data_reporting, 119, 6);
    define_bitflag_getter!(cap_feature_set_epc, 119, 7);
    define_bitflag_getter!(cap_feature_set_accessible_max_addr_config, 119, 8);
    define_bitflag_getter!(cap_feature_set_dsn, 119, 9);

    // word 120
    define_bitflag_getter!(enabled_feature_set_write_read_verify, 120, 1);
    define_bitflag_getter!(enabled_cmd_write_uncorrectable_ext, 120, 2);
    define_bitflag_getter!(enabled_cmd_read_log_dma_ext, 120, 3);
    define_bitflag_getter!(enabled_feature_set_free_fall, 120, 5);
    define_bitflag_getter!(enabled_feature_set_sense_data_reporting, 120, 6);
    define_bitflag_getter!(enabled_feature_set_epc, 120, 7);
    define_bitflag_getter!(enabled_feature_set_accessible_max_addr_config, 120, 8);
    define_bitflag_getter!(enabled_feature_set_dsn, 120, 9);

    // word 128
    define_bitflag_getter!(security_supported, 128, 0);
    define_bitflag_getter!(security_enabled, 128, 1);
    define_bitflag_getter!(security_locked, 128, 2);
    define_bitflag_getter!(security_frozen, 128, 3);
    define_bitflag_getter!(security_count_expired, 128, 4);
    define_bitflag_getter!(security_enhanced_security_erase_supported, 128, 5);
    define_bitflag_getter!(security_master_password_capability, 128, 8);

    pub fn security_state(&self) -> SecurityState {
        let a = self.security_supported();
        let b = self.security_enabled();
        let c = self.security_locked();
        let d = self.security_frozen();
        let e = self.security_count_expired();
        let f = self.security_master_password_capability();
        match (a, b, c, d, e, f) {
            // 0 is a transitional state
            (true, false, false, false, _, false) => SecurityState::SEC1,
            (true, false, false, true, _, _) => SecurityState::SEC2,
            // 3 is a transitional state
            (true, true, true, false, _, _) => SecurityState::SEC4,
            (true, true, false, false, _, _) => SecurityState::SEC5,
            (true, true, false, true, _, _) => SecurityState::SEC6,
            _ => SecurityState::Unknown,
        }
    }

    pub fn time_required_for_normal_erase(&self) -> Duration {
        let w89 = self.word(89);
        match (w89 >> 15) & 1 {
            1 => Duration::from_mins((w89 & 0x7FFF) as u64 * 2),
            _ => Duration::from_mins((w89 & 0x00FF) as u64 * 2),
        }
    }

    pub fn time_required_for_enhanced_erase(&self) -> Duration {
        let w90 = self.word(90);
        match (w90 >> 15) & 1 {
            1 => Duration::from_mins((w90 & 0x7FFF) as u64 * 2),
            _ => Duration::from_mins((w90 & 0x00FF) as u64 * 2),
        }
    }

    pub fn capacity_lba(&self) -> u64 {
        let mut buf = [0u8; 8];
        if self.cap_lba48() {
            buf.copy_from_slice(&self.bfr[100 * 2..100 * 2 + 8]);
            u64::from_le_bytes(buf)
        } else if self.cap_lba28() {
            buf[..4].copy_from_slice(&self.bfr[60 * 2..60 * 2 + 4]);
            u64::from_le_bytes(buf)
        } else {
            0
        }
    }

    pub fn capacity_bytes(&self) -> u64 {
        self.capacity_lba() * self.logical_sector_size()
    }

    /// Logical sector size in bytes
    pub fn logical_sector_size(&self) -> u64 {
        let w106 = self.word(106);
        // Bit 15 = 0 → 512-byte logical sectors
        if w106 & (1 << 15) == 0 {
            return 512;
        }
        // Bits 11:0 = exponent offset
        let exponent = (w106 & 0x0FFF) as u32;
        // Logical size = 2^(exponent + 9)
        1u64 << (exponent + 9)
    }

    /// Physical sector size in bytes
    pub fn physical_sector_size(&self) -> u64 {
        let w106 = self.word(106);
        let logical = self.logical_sector_size();
        // Bit 14 = 0 → physical = logical
        if w106 & (1 << 14) == 0 {
            return logical;
        }
        // Bits 11:0 = number of logical sectors per physical sector (power of two)
        let exponent = (w106 & 0x0FFF) as u32;
        logical * (1u64 << exponent)
    }
}

impl ScsiSat<'_> {
    /// IDENTIFY DEVICE (0xEC)
    pub fn identify(&self) -> IdentifySatCommand<'_> {
        IdentifySatCommand::new(self)
    }
}
