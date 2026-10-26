/// Command opcodes understood by RadiaCode firmware.
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Command {
    /// Device status flags (`u32`).
    GetStatus = 0x0005,
    /// Session handshake; sent first on every connect.
    SetExchange = 0x0007,
    /// Boot and target firmware versions.
    GetVersion = 0x000A,
    /// Hardware serial number as hex groups.
    GetSerial = 0x000B,
    /// Firmware image metadata.
    FwImageGetInfo = 0x0012,
    /// Firmware signature, file name and id string.
    FwSignature = 0x0101,
    /// Hardware configuration block.
    RdHwConfig = 0x0807,
    /// Raw flash read.
    RdFlash = 0x081C,
    /// Read one virtual special-function register.
    RdVirtSfr = 0x0824,
    /// Write one virtual special-function register.
    WrVirtSfr = 0x0825,
    /// Read a virtual string.
    RdVirtString = 0x0826,
    /// Write a virtual string.
    WrVirtString = 0x0827,
    /// Read up to 32 registers in one request.
    RdVirtSfrBatch = 0x082A,
    /// Write up to 32 registers in one request.
    WrVirtSfrBatch = 0x082B,
    /// Set the device local clock.
    SetTime = 0x0A04,
}

/// Every command opcode known to this crate.
pub const ALL_COMMANDS: [Command; 15] = [
    Command::GetStatus,
    Command::SetExchange,
    Command::GetVersion,
    Command::GetSerial,
    Command::FwImageGetInfo,
    Command::FwSignature,
    Command::RdHwConfig,
    Command::RdFlash,
    Command::RdVirtSfr,
    Command::WrVirtSfr,
    Command::RdVirtString,
    Command::WrVirtString,
    Command::RdVirtSfrBatch,
    Command::WrVirtSfrBatch,
    Command::SetTime,
];

impl Command {
    /// Looks up a command by its wire opcode.
    pub fn from_opcode(opcode: u16) -> Option<Self> {
        ALL_COMMANDS.into_iter().find(|command| u16::from(*command) == opcode)
    }
}

impl From<Command> for u16 {
    fn from(value: Command) -> Self {
        value as u16
    }
}

/// Virtual string identifiers readable with [`Command::RdVirtString`].
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum VirtString {
    /// Device configuration INI, CP1251 encoded.
    Configuration = 2,
    /// Firmware descriptor text.
    FwDescriptor = 3,
    /// User-visible serial number, e.g. `RC-103-001234`.
    SerialNumber = 8,
    /// Device text message (ASCII).
    TextMessage = 0x0F,
    /// Memory snapshot blob.
    MemSnapshot = 0xE0,
    /// Ring buffer of monitor records (rates, dose, events).
    DataBuf = 0x100,
    /// Self-describing register directory (`SFR_FILE`).
    SfrFile = 0x101,
    /// Live spectrum histogram.
    Spectrum = 0x200,
    /// Energy calibration coefficients `[a0, a1, a2]`.
    EnergyCalib = 0x202,
    /// Accumulated spectrum histogram.
    SpecAccum = 0x205,
    /// Differential spectrum histogram.
    SpecDiff = 0x206,
    /// Spectrum reset channel.
    SpecReset = 0x207,
}

impl From<VirtString> for u32 {
    fn from(value: VirtString) -> Self {
        value as u32
    }
}

#[cfg(test)]
mod tests {
    use super::{Command, VirtString, ALL_COMMANDS};

    #[test]
    fn opcodes_round_trip() {
        ALL_COMMANDS.into_iter().for_each(|command| {
            assert_eq!(Command::from_opcode(u16::from(command)), Some(command));
        });
        assert_eq!(Command::from_opcode(0xFFFF), None);
    }

    #[test]
    fn wire_values_match_reference() {
        assert_eq!(u16::from(Command::RdVirtString), 0x0826);
        assert_eq!(u32::from(VirtString::DataBuf), 0x100);
        assert_eq!(u32::from(VirtString::Spectrum), 0x200);
    }
}
