use std::fs;

use rusb::{Context, UsbContext};

use super::blocking::run_blocking;
use super::error::UsbError;
use super::open::{is_radiacode_device, open_handle};
use crate::error::Result;

/// udev rule that grants non-root access to RadiaCode devices on Linux.
///
/// Install it as [`UDEV_RULE_PATH`], then run `udevadm control --reload &&
/// udevadm trigger` and replug the device. The same rule ships as
/// `radiacode.rules` in the crate root.
pub const UDEV_RULE: &str = include_str!("../../radiacode.rules");

/// Conventional install path for [`UDEV_RULE`].
pub const UDEV_RULE_PATH: &str = "/etc/udev/rules.d/99-radiacode.rules";

/// Whether the current process can open attached RadiaCode devices.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbAccessStatus {
    /// Access works, or no device is attached so there is nothing to diagnose.
    Granted,
    /// Access is denied and no udev rule granting it is installed.
    RuleMissing,
    /// A rule is installed but the device was attached before it took effect; replug it.
    RuleInstalledNeedReplug,
}

/// Probes whether an attached RadiaCode can be opened and, if not, why.
pub async fn access_status() -> Result<UsbAccessStatus> {
    run_blocking(|| Ok(access_status_blocking())).await
}

/// Returns `true` when [`UDEV_RULE_PATH`] exists and grants access.
pub fn rule_installed() -> bool {
    fs::read_to_string(UDEV_RULE_PATH).is_ok_and(|content| rule_grants_access(&content))
}

/// Returns `true` when udev rule text grants world access to the RadiaCode VID/PID.
pub fn rule_grants_access(content: &str) -> bool {
    let lowered = content.to_ascii_lowercase();
    ["0483", "f123", "0666"].iter().all(|token| lowered.contains(token))
}

pub(super) fn status_for_denied_access(rule_present: bool) -> UsbAccessStatus {
    match rule_present {
        true => UsbAccessStatus::RuleInstalledNeedReplug,
        false => UsbAccessStatus::RuleMissing,
    }
}

fn access_status_blocking() -> UsbAccessStatus {
    match Context::new() {
        Ok(context) if device_present(&context) && permission_denied(&context) => {
            status_for_denied_access(rule_installed())
        }
        _ => UsbAccessStatus::Granted,
    }
}

fn device_present(context: &Context) -> bool {
    context.devices().is_ok_and(|devices| devices.iter().any(|device| is_radiacode_device(&device)))
}

fn permission_denied(context: &Context) -> bool {
    matches!(open_handle(context, None), Err(UsbError::PermissionDenied | UsbError::Usb(rusb::Error::Access)))
}
