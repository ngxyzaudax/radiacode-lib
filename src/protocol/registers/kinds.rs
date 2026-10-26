use super::VirtSfr;

/// How a 32-bit register word should be interpreted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VsfrValueKind {
    /// Unsigned byte stored in the low or high byte of the word.
    U8,
    /// Unsigned halfword stored in the low or high half of the word.
    U16,
    /// Signed halfword stored in the low or high half of the word.
    I16,
    /// Full unsigned word.
    U32,
    /// Full signed word.
    I32,
    /// IEEE-754 single-precision float.
    F32,
    /// Boolean stored as a byte.
    Bool,
}

const CATALOG: [VirtSfr; 29] = [
    VirtSfr::DeviceCtrl,
    VirtSfr::DeviceTime,
    VirtSfr::DispBrt,
    VirtSfr::DispOffTime,
    VirtSfr::DispDir,
    VirtSfr::SoundCtrl,
    VirtSfr::SoundOn,
    VirtSfr::VibroCtrl,
    VirtSfr::VibroOn,
    VirtSfr::LedsOn,
    VirtSfr::AlarmMode,
    VirtSfr::DrLev1UrH,
    VirtSfr::DrLev2UrH,
    VirtSfr::DsUnits,
    VirtSfr::CpsFilter,
    VirtSfr::RawFilter,
    VirtSfr::DoseReset,
    VirtSfr::CrLev1Cp10s,
    VirtSfr::CrLev2Cp10s,
    VirtSfr::CrUnits,
    VirtSfr::DsLev1Ur,
    VirtSfr::DsLev2Ur,
    VirtSfr::Cps,
    VirtSfr::DrUrH,
    VirtSfr::DsUr,
    VirtSfr::TempDegC,
    VirtSfr::VBiasMv,
    VirtSfr::SysStatus,
    VirtSfr::SysMcuTemp,
];

/// Registers whose layout is validated against the device `SFR_FILE` on connect.
pub fn catalog_registers() -> &'static [VirtSfr] {
    &CATALOG
}

pub(super) fn static_value_kind(register: VirtSfr) -> VsfrValueKind {
    use VirtSfr::*;
    match register {
        DispBrt | SoundVol | SoundOn | VibroOn | Led0Brt | Led1Brt | Led2Brt | Led3Brt | LedsOn | CpsFilter
        | RawFilter | DsUnits | CrUnits | UseNanoSvH | DispCtrl | DispContr | DispDir | BleTxPwr | PlaySignal
        | MsCtrl | MsMode | MsSubMode | DpotRdac | DpotRdacEeprom | DpotToler => VsfrValueKind::U8,
        DispOn | DispBacklightOn | DeviceLang | DeviceOn | MsRun | DoseReset | CalibMode => VsfrValueKind::Bool,
        SysRxSize | SysTxSize | Opt | VBiasMv => VsfrValueKind::U16,
        AccX | AccY | AccZ | CompLev => VsfrValueKind::I16,
        ChnToKeVA0 | ChnToKeVA1 | ChnToKeVA2 | TempDegC | RawTempDegC | TempUpDegC | TempDnDegC | SysMcuTemp => {
            VsfrValueKind::F32
        }
        SysMcuVref => VsfrValueKind::I32,
        _ => VsfrValueKind::U32,
    }
}
