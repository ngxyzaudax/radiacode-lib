/// One `[CHN_*]` channel definition inside a message group.
#[derive(Debug, Clone, PartialEq)]
pub struct ChannelDef {
    /// Channel id (`Id=`).
    pub id: u8,
    /// Display unit (`Unit=`), decoded from CP1251.
    pub unit: String,
    /// Linear scale (`P1=`, default `1.0`).
    pub scale: f64,
    /// Linear offset (`P2=`, default `0.0`).
    pub offset: f64,
}

impl Default for ChannelDef {
    fn default() -> Self {
        Self { id: 0, unit: String::new(), scale: 1.0, offset: 0.0 }
    }
}

/// One `[[GRP_*]]` message group of the device configuration.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MessageGroup {
    /// Section name without brackets, for example `GRP_RealTimeData`.
    pub name: String,
    /// Entity id (`Id=` before any channel).
    pub entity: u8,
    /// Group id (`GId=`).
    pub group: u8,
    /// Channels declared in the group.
    pub channels: Vec<ChannelDef>,
}

/// Parsed device `Configuration` virtual string.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigurationCatalog {
    /// Spectrum encoding version (`SpecFormatVersion=`).
    pub spec_format_version: u32,
    /// Message groups in declaration order.
    pub groups: Vec<MessageGroup>,
}

#[derive(Default)]
struct IniParseState {
    catalog: ConfigurationCatalog,
    group: Option<MessageGroup>,
    channel: Option<ChannelDef>,
}

impl IniParseState {
    fn apply_line(self, line: &str) -> Self {
        match line {
            "" => self,
            _ if line.starts_with("[[GRP_") && line.ends_with(']') => self.open_group(line),
            _ if line.starts_with("[CHN_") && line.ends_with(']') => self.open_channel(),
            _ => match line.strip_prefix("SpecFormatVersion=") {
                Some(value) => self.with_spec_format(value),
                None => self.with_field(line),
            },
        }
    }

    fn with_spec_format(mut self, value: &str) -> Self {
        self.catalog.spec_format_version = value.parse().unwrap_or(0);
        self
    }

    fn open_group(self, line: &str) -> Self {
        let name = line.trim_start_matches("[[").trim_end_matches(']');
        let catalog = self.close_group();
        Self { catalog, group: Some(MessageGroup { name: name.to_string(), ..MessageGroup::default() }), channel: None }
    }

    fn open_channel(self) -> Self {
        Self { group: self.close_channel(), channel: Some(ChannelDef::default()), ..self }
    }

    fn with_field(self, line: &str) -> Self {
        let has_channel = self.channel.is_some();
        Self {
            channel: self.channel.map(|channel| apply_channel_field(channel, line)),
            group: self.group.map(|group| apply_group_field(group, line, has_channel)),
            ..self
        }
    }

    fn close_channel(&self) -> Option<MessageGroup> {
        self.group.clone().map(|group| MessageGroup {
            channels: group.channels.into_iter().chain(self.channel.clone()).collect(),
            ..group
        })
    }

    fn close_group(self) -> ConfigurationCatalog {
        let group = self.close_channel();
        ConfigurationCatalog { groups: self.catalog.groups.into_iter().chain(group).collect(), ..self.catalog }
    }
}

fn apply_channel_field(channel: ChannelDef, line: &str) -> ChannelDef {
    match line.split_once('=') {
        Some(("Id", value)) => ChannelDef { id: value.parse().unwrap_or(0), ..channel },
        Some(("Unit", value)) => ChannelDef { unit: value.to_string(), ..channel },
        Some(("P1", value)) => ChannelDef { scale: value.parse().unwrap_or(1.0), ..channel },
        Some(("P2", value)) => ChannelDef { offset: value.parse().unwrap_or(0.0), ..channel },
        _ => channel,
    }
}

fn apply_group_field(group: MessageGroup, line: &str, has_channel: bool) -> MessageGroup {
    match line.split_once('=') {
        Some(("Id", value)) if !has_channel => MessageGroup { entity: value.parse().unwrap_or(0), ..group },
        Some(("GId", value)) => MessageGroup { group: value.parse().unwrap_or(0), ..group },
        _ => group,
    }
}

/// Parses the device `Configuration` INI text (already decoded from CP1251).
pub fn parse_configuration_ini(text: &str) -> ConfigurationCatalog {
    text.lines().map(str::trim).fold(IniParseState::default(), IniParseState::apply_line).close_group()
}
