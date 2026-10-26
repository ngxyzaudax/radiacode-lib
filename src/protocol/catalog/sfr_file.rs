/// Value type declared by the device `SFR_FILE` (`Type=` field).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum SfrEntryKind {
    /// `Type=1`.
    U8,
    /// `Type=2`.
    U16,
    /// `Type=4`.
    U32,
    /// Signed word; reserved for firmware that reports it explicitly.
    I32,
    /// `Type=5`.
    F32,
    /// Boolean stored as a byte; reserved for firmware that reports it explicitly.
    Bool,
    /// Any other or missing `Type=` value.
    Unknown,
}

/// One `[VSFR_*]` section of the device `SFR_FILE` register directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SfrCatalogEntry {
    /// Section name, for example `VSFR_DISP_BRT`.
    pub name: String,
    /// Register address (`Addr=`, hexadecimal).
    pub address: u32,
    /// Register size in bytes (`Size=`).
    pub size: u8,
    /// Declared value type (`Type=`).
    pub value_kind: SfrEntryKind,
    /// Whether the register is signed (`Signed=1`).
    pub signed: bool,
}

impl SfrCatalogEntry {
    fn named(name: &str) -> Self {
        Self { name: name.to_string(), address: 0, size: 0, value_kind: SfrEntryKind::Unknown, signed: false }
    }

    fn with_field(self, line: &str) -> Self {
        match line.split_once('=') {
            Some(("Addr", value)) => Self { address: parse_hex_address(value), ..self },
            Some(("Size", value)) => Self { size: value.trim().parse().unwrap_or(0), ..self },
            Some(("Type", value)) => Self { value_kind: parse_value_kind(value.trim()), ..self },
            Some(("Signed", value)) => Self { signed: value.trim() == "1", ..self },
            _ => self,
        }
    }
}

#[derive(Default)]
struct SfrParseState {
    entries: Vec<SfrCatalogEntry>,
    current: Option<SfrCatalogEntry>,
}

impl SfrParseState {
    fn apply_line(self, line: &str) -> Self {
        match section_name(line) {
            Some(section) => self.open_section(section),
            None => Self { current: self.current.map(|entry| entry.with_field(line)), ..self },
        }
    }

    fn open_section(self, section: &str) -> Self {
        let next = section.contains("VSFR_").then(|| SfrCatalogEntry::named(section));
        Self { current: next, entries: self.finish() }
    }

    fn finish(self) -> Vec<SfrCatalogEntry> {
        self.entries.into_iter().chain(self.current).collect()
    }
}

/// Parses the device `SFR_FILE` register directory (INI-like text).
///
/// Only `[VSFR_*]` sections produce entries; any other section closes the
/// current entry. Unparseable fields fall back to zero / [`SfrEntryKind::Unknown`].
pub fn parse_sfr_file(text: &str) -> Vec<SfrCatalogEntry> {
    text.lines().map(str::trim).fold(SfrParseState::default(), SfrParseState::apply_line).finish()
}

fn section_name(line: &str) -> Option<&str> {
    line.strip_prefix('[')?.strip_suffix(']')
}

fn parse_value_kind(value: &str) -> SfrEntryKind {
    match value {
        "1" => SfrEntryKind::U8,
        "2" => SfrEntryKind::U16,
        "4" => SfrEntryKind::U32,
        "5" => SfrEntryKind::F32,
        _ => SfrEntryKind::Unknown,
    }
}

fn parse_hex_address(value: &str) -> u32 {
    let trimmed = value.trim().trim_start_matches("0x").trim_start_matches("0X");
    u32::from_str_radix(trimmed, 16).unwrap_or(0)
}
