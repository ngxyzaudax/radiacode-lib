/// Link RSSI, or why a sample could not be taken.
///
/// `PermissionDenied` means the process could not query the BlueZ management
/// socket. That socket requires `CAP_NET_ADMIN`. Membership in the `bluetooth`
/// group is enough for `bluetoothctl` over D-Bus and is not enough for this query.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RssiStatus {
    /// RSSI in dBm.
    Measured(i16),
    /// The operating system refused the RSSI query.
    PermissionDenied,
    /// The link did not report an RSSI value.
    #[default]
    NotReported,
}

impl RssiStatus {
    /// Returns the measured dBm value when this status is [`RssiStatus::Measured`].
    pub fn dbm(self) -> Option<i16> {
        match self {
            Self::Measured(value) => Some(value),
            Self::PermissionDenied | Self::NotReported => None,
        }
    }

    /// Keeps `previous` when this sample has nothing new to report.
    pub fn prefer_over(self, previous: Self) -> Self {
        match self {
            Self::NotReported => previous,
            other => other,
        }
    }
}

impl std::fmt::Display for RssiStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Measured(value) => write!(formatter, "{value} dBm"),
            Self::PermissionDenied => formatter.write_str("RSSI unavailable: permission denied"),
            Self::NotReported => formatter.write_str("RSSI unavailable"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RssiStatus;

    #[test]
    fn permission_denied_is_distinct_from_a_missing_sample() {
        assert_eq!(RssiStatus::PermissionDenied.to_string(), "RSSI unavailable: permission denied");
        assert_eq!(RssiStatus::PermissionDenied.dbm(), None);
        assert_eq!(RssiStatus::NotReported.to_string(), "RSSI unavailable");
        assert_eq!(RssiStatus::NotReported.prefer_over(RssiStatus::Measured(-70)), RssiStatus::Measured(-70));
        assert_eq!(RssiStatus::PermissionDenied.prefer_over(RssiStatus::Measured(-70)), RssiStatus::PermissionDenied);
        assert_eq!(RssiStatus::Measured(-55).dbm(), Some(-55));
    }
}
