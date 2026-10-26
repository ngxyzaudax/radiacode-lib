mod extract;
mod kinds;
mod table;

#[cfg(test)]
mod tests;

pub use extract::{extract_vsfr_f32, extract_vsfr_u32, resolve_value_kind};
pub use kinds::{catalog_registers, VsfrValueKind};
pub use table::ALL_REGISTERS;

/// Virtual special-function registers (VSFR) exposed by the firmware.
///
/// Addresses mirror the device `SFR_FILE` directory. Service and calibration
/// registers are listed for completeness; this crate only writes the user
/// settings registers through high-level helpers.
#[allow(missing_docs)]
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum VirtSfr {
    DeviceCtrl = 0x0500,
    DeviceLang = 0x0502,
    DeviceOn = 0x0503,
    DeviceTime = 0x0504,
    DispCtrl = 0x0510,
    DispBrt = 0x0511,
    DispContr = 0x0512,
    DispOffTime = 0x0513,
    DispOn = 0x0514,
    DispDir = 0x0515,
    DispBacklightOn = 0x0516,
    SoundCtrl = 0x0520,
    SoundVol = 0x0521,
    SoundOn = 0x0522,
    SoundButton = 0x0523,
    VibroCtrl = 0x0530,
    VibroOn = 0x0531,
    LedsCtrl = 0x0540,
    Led0Brt = 0x0541,
    Led1Brt = 0x0542,
    Led2Brt = 0x0543,
    Led3Brt = 0x0544,
    LedsOn = 0x0545,
    AlarmMode = 0x05E0,
    PlaySignal = 0x05E1,
    MsCtrl = 0x0600,
    MsMode = 0x0601,
    MsSubMode = 0x0602,
    MsRun = 0x0603,
    BleTxPwr = 0x0700,
    DrLev1UrH = 0x8000,
    DrLev2UrH = 0x8001,
    DsLev1HundredUr = 0x8002,
    DsLev2HundredUr = 0x8003,
    DsUnits = 0x8004,
    CpsFilter = 0x8005,
    RawFilter = 0x8006,
    DoseReset = 0x8007,
    CrLev1Cp10s = 0x8008,
    CrLev2Cp10s = 0x8009,
    UseNanoSvH = 0x800C,
    ChnToKeVA0 = 0x8010,
    ChnToKeVA1 = 0x8011,
    ChnToKeVA2 = 0x8012,
    CrUnits = 0x8013,
    DsLev1Ur = 0x8014,
    DsLev2Ur = 0x8015,
    Cps = 0x8020,
    DrUrH = 0x8021,
    DsUr = 0x8022,
    TempDegC = 0x8024,
    AccX = 0x8025,
    AccY = 0x8026,
    AccZ = 0x8027,
    Opt = 0x8028,
    RawTempDegC = 0x8033,
    TempUpDegC = 0x8034,
    TempDnDegC = 0x8035,
    VBiasMv = 0xC000,
    CompLev = 0xC001,
    CalibMode = 0xC002,
    DpotRdac = 0xC004,
    DpotRdacEeprom = 0xC005,
    DpotToler = 0xC006,
    SysMcuId0 = 0xFFFF_0000,
    SysMcuId1 = 0xFFFF_0001,
    SysMcuId2 = 0xFFFF_0002,
    SysDeviceId = 0xFFFF_0005,
    SysSignature = 0xFFFF_0006,
    SysRxSize = 0xFFFF_0007,
    SysTxSize = 0xFFFF_0008,
    SysBootVersion = 0xFFFF_0009,
    SysTargetVersion = 0xFFFF_000A,
    SysStatus = 0xFFFF_000B,
    SysMcuVref = 0xFFFF_000C,
    SysMcuTemp = 0xFFFF_000D,
}

impl VirtSfr {
    /// Wire address of the register.
    pub fn address(self) -> u32 {
        self as u32
    }

    /// Looks up a register by wire address.
    pub fn from_address(address: u32) -> Option<Self> {
        ALL_REGISTERS.into_iter().find(|register| register.address() == address)
    }

    /// Value kind assumed when the device `SFR_FILE` does not describe the register.
    pub fn static_value_kind(self) -> VsfrValueKind {
        kinds::static_value_kind(self)
    }
}

impl From<VirtSfr> for u32 {
    fn from(value: VirtSfr) -> Self {
        value.address()
    }
}

impl TryFrom<u32> for VirtSfr {
    type Error = u32;

    fn try_from(address: u32) -> Result<Self, Self::Error> {
        Self::from_address(address).ok_or(address)
    }
}
