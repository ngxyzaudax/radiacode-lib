use std::fmt;

const DEFAULT_LABEL: &str = "RadiaCode";

/// Physical link type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TransportKind {
    /// Bluetooth Low Energy.
    Bluetooth,
    /// USB (vendor-specific bulk interface).
    Usb,
}

impl TransportKind {
    /// Human-readable name, `Bluetooth` or `USB`.
    pub fn label(self) -> &'static str {
        match self {
            Self::Bluetooth => "Bluetooth",
            Self::Usb => "USB",
        }
    }
}

impl fmt::Display for TransportKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.label())
    }
}

/// Address of a device on a specific transport.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DeviceEndpoint {
    /// Bluetooth LE peripheral, addressed by MAC address (or platform peripheral id on macOS).
    Bluetooth {
        /// MAC address such as `aa:bb:cc:dd:ee:ff`.
        address: String,
    },
    /// USB device, addressed by serial number or by `usb-<index>` when the serial is unreadable.
    Usb {
        /// USB serial number string.
        serial: String,
    },
}

impl DeviceEndpoint {
    /// Transport the endpoint belongs to.
    pub fn transport(&self) -> TransportKind {
        match self {
            Self::Bluetooth { .. } => TransportKind::Bluetooth,
            Self::Usb { .. } => TransportKind::Usb,
        }
    }

    /// The address or serial that identifies the endpoint.
    pub fn address_label(&self) -> &str {
        match self {
            Self::Bluetooth { address } => address,
            Self::Usb { serial } => serial,
        }
    }
}

impl fmt::Display for DeviceEndpoint {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {}", self.transport(), self.address_label())
    }
}

/// A device found by a scan.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DiscoveredDevice {
    /// Where to connect.
    pub endpoint: DeviceEndpoint,
    /// Label reported by the scan (model, serial or advertised name).
    pub label: String,
    /// Device serial number, such as `RC-110-006806`, when known.
    pub serial: Option<String>,
    /// Device model, such as `RC-110`, when known.
    pub model: Option<String>,
    /// Advertisement RSSI in dBm (Bluetooth only).
    pub rssi: Option<i16>,
}

impl DiscoveredDevice {
    /// Best available label: the scan label, then model, then serial, then `RadiaCode`.
    pub fn display_label(&self) -> String {
        Some(self.label.clone())
            .filter(|label| !label.is_empty())
            .or_else(|| self.model.clone())
            .or_else(|| self.serial.clone())
            .unwrap_or_else(|| DEFAULT_LABEL.into())
    }
}
