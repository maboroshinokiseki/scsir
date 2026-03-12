use crate::command::ata::{raw::RawSatCommand, AtaProtocol, SatResult, ScsiSat, ToDevice};

const OPCODE_SECURITY_SET_PASSWORD: u8 = 0xF1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SecurityPassword {
    /// User Password (supported by all drives that support the SECURITY feature set)
    User,
    /// Master Password
    Master,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SecurityMode {
    /// Enables the drive’s *High* security level.
    ///
    /// In High mode, the drive can be unlocked with either the **user**
    /// password or the **master** password.
    ///
    /// This mode allows the master password to act as a fallback
    /// recovery mechanism: if the user password is lost, the master
    /// password may still unlock or disable security, depending on the
    /// drive’s configuration.
    ///
    /// High mode provides strong protection while still permitting
    /// administrative recovery.
    High,

    /// Enables the drive’s *Maximum* security level.
    ///
    /// In Maximum mode, only the **user** password can unlock the drive.
    /// The master password can no longer unlock or disable security;
    /// it may only be used to perform a *Security Erase Unit* operation.
    ///
    /// This mode removes the administrative recovery path and ensures
    /// that data cannot be accessed without the user password.
    ///
    /// Maximum mode offers the strongest protection defined by the ATA
    /// Security Feature Set, but at the cost of losing master‑password
    /// unlock capability.
    Maximum,
}

#[derive(Clone, Debug)]
pub struct SecuritySetPasswordSatCommand<'a> {
    raw: RawSatCommand<'a, ToDevice>,
    data: Vec<u8>,
}

impl<'a> SecuritySetPasswordSatCommand<'a> {
    fn new(sat: &'a ScsiSat<'a>) -> Self {
        let mut raw = sat.raw_write();
        raw.command(AtaProtocol::PioDataOut, OPCODE_SECURITY_SET_PASSWORD);
        Self {
            raw,
            data: vec![0u8; 512],
        }
    }

    pub fn mode(&mut self, level: SecurityMode) -> &mut Self {
        self.data[1] = match level {
            SecurityMode::High => 0x00,
            SecurityMode::Maximum => 0x01,
        };
        self
    }

    pub fn password(&mut self, which: SecurityPassword, password: &[u8]) -> &mut Self {
        assert!(password.len() <= 32);
        self.data[0] = match which {
            SecurityPassword::User => 0x00,
            SecurityPassword::Master => 0x01,
        };
        self.data[2..2 + password.len()].copy_from_slice(password);
        self
    }

    pub fn device(&mut self, device: u8) -> &mut Self {
        self.raw.device(device);
        self
    }

    // ######################################################################

    pub fn issue_12(&mut self) -> SatResult<()> {
        self.raw.parameter(&self.data).issue_12()
    }

    pub fn issue_16(&mut self) -> SatResult<()> {
        self.raw.parameter(&self.data).issue_16()
    }
}

// ######################################################################

impl ScsiSat<'_> {
    /// Command: SECURITY SET PASWORD (0xF1)
    pub fn security_set_password(&self) -> SecuritySetPasswordSatCommand<'_> {
        SecuritySetPasswordSatCommand::new(self)
    }
}
