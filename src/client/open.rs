use std::time::{Duration, SystemTime};

use tracing::{debug, info, warn};

use super::{OpenOptions, RadiaCode};
use crate::error::{Error, Result};
use crate::protocol::Sequence;
use crate::transport::BoxedTransport;

const CONNECT_ATTEMPTS: usize = 3;
const RECONNECT_ATTEMPTS: usize = 2;
const CONNECT_RETRY_DELAY: Duration = Duration::from_millis(1500);
const RECONNECT_RETRY_DELAY: Duration = Duration::from_millis(3000);

type OpenFailure = (Error, BoxedTransport);

fn retry_policy(options: &OpenOptions) -> (usize, Duration) {
    match options.restore {
        Some(_) => (RECONNECT_ATTEMPTS, RECONNECT_RETRY_DELAY),
        None => (CONNECT_ATTEMPTS, CONNECT_RETRY_DELAY),
    }
}

async fn disconnect_quietly(transport: BoxedTransport) {
    if let Err(error) = transport.disconnect().await {
        debug!(%error, "transport disconnect after failed open reported an error");
    }
}

impl RadiaCode {
    /// Initialises a session on `transport`.
    ///
    /// Performs the exchange handshake, synchronises the device clock, reads
    /// the register directory, firmware version and spectrum format. Transient
    /// failures are retried on the same transport; on final failure the
    /// transport is disconnected.
    pub async fn open(transport: BoxedTransport, options: OpenOptions) -> Result<Self> {
        let (attempts, delay) = retry_policy(&options);
        info!(
            ignore_firmware_check = options.ignore_firmware_check,
            reconnect = options.restore.is_some(),
            "opening radiacode session"
        );
        let mut transport = transport;
        let mut last_error = Error::ConnectionClosed;
        for attempt in 0..attempts {
            if attempt > 0 {
                warn!(attempt, error = %last_error, "retrying radiacode open after transient failure");
                tokio::time::sleep(delay).await;
            }
            match Self::open_once(transport, &options).await {
                Ok(device) => return Ok(device),
                Err((error, returned)) if error.is_transient() => (transport, last_error) = (returned, error),
                Err((error, returned)) => {
                    disconnect_quietly(returned).await;
                    return Err(error);
                }
            }
        }
        disconnect_quietly(transport).await;
        Err(last_error)
    }

    fn new_session(transport: BoxedTransport, options: &OpenOptions) -> Self {
        let restore = options.restore.as_ref();
        Self {
            transport,
            sequence: Sequence::session_start(),
            spectrum_format_version: restore.map_or(0, |session| session.spectrum_format_version),
            versions: restore.map(|session| session.versions.clone()),
            base_time: SystemTime::now(),
            sfr_entries: Vec::new(),
        }
    }

    async fn open_once(transport: BoxedTransport, options: &OpenOptions) -> std::result::Result<Self, OpenFailure> {
        let mut device = Self::new_session(transport, options);
        debug!(seq_start = device.sequence.start_value(), "command sequence initialized");
        match device.initialize(options).await {
            Ok(()) => Ok(device),
            Err(error) => {
                warn!(%error, "session initialization failed");
                Err((error, device.transport))
            }
        }
    }
}
