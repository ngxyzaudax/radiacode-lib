use std::time::Duration;

use tracing::warn;

use super::RadiaCode;
use crate::error::{Error, Result};
use crate::protocol::{
    decode_batch_read, decode_batch_words, decode_batch_write, encode_batch_read_args, encode_batch_write_args,
    validate_batch_size, BatchReadResponse, Command, VirtSfr,
};

const BATCH_RETRIES: usize = 2;
const BATCH_RETRY_DELAY: Duration = Duration::from_millis(100);

impl RadiaCode {
    /// Reads up to 32 registers in one request and extracts typed values.
    ///
    /// Transient failures are retried; if the batch command keeps failing the
    /// registers are read one by one. Registers missing from a partial batch
    /// response are read individually.
    pub async fn read_vsfr_batch(&mut self, ids: &[VirtSfr]) -> Result<Vec<u32>> {
        validate_batch_size(ids.len())?;
        let raw = self.read_batch_with_retries(ids).await?;
        Ok(decode_batch_words(ids, &raw, &self.sfr_entries))
    }

    /// Reads up to 32 registers in one request without retries or value extraction.
    pub async fn read_vsfr_batch_raw(&mut self, ids: &[VirtSfr]) -> Result<Vec<u32>> {
        validate_batch_size(ids.len())?;
        self.read_batch_once(ids).await
    }

    /// Writes up to 32 `(register, value)` pairs in one request, retrying transient failures.
    pub async fn write_vsfr_batch(&mut self, pairs: &[(VirtSfr, u32)]) -> Result<()> {
        validate_batch_size(pairs.len())?;
        let mut attempt = 0;
        loop {
            match self.write_batch_once(pairs).await {
                Err(error) if error.is_transient() && attempt < BATCH_RETRIES => {
                    warn!(attempt, %error, count = pairs.len(), "vsfr batch write failed, retrying");
                    attempt += 1;
                    tokio::time::sleep(BATCH_RETRY_DELAY).await;
                }
                outcome => return outcome,
            }
        }
    }

    async fn write_batch_once(&mut self, pairs: &[(VirtSfr, u32)]) -> Result<()> {
        let response = self.execute_raw(Command::WrVirtSfrBatch, &encode_batch_write_args(pairs)).await?;
        decode_batch_write(response, pairs.len())
    }

    async fn read_batch_with_retries(&mut self, ids: &[VirtSfr]) -> Result<Vec<u32>> {
        let mut attempt = 0;
        loop {
            match self.read_batch_once(ids).await {
                Err(error) if error.is_transient() && attempt < BATCH_RETRIES => {
                    warn!(attempt, %error, count = ids.len(), "vsfr batch read failed, retrying");
                    attempt += 1;
                    tokio::time::sleep(BATCH_RETRY_DELAY).await;
                }
                Err(error) if falls_back_to_single_reads(&error) => {
                    warn!(%error, count = ids.len(), "vsfr batch read failed, falling back to sequential reads");
                    return self.read_each(ids).await;
                }
                outcome => return outcome,
            }
        }
    }

    async fn read_batch_once(&mut self, ids: &[VirtSfr]) -> Result<Vec<u32>> {
        let response = self.execute_raw(Command::RdVirtSfrBatch, &encode_batch_read_args(ids)).await?;
        match decode_batch_read(response, ids.len())? {
            BatchReadResponse::Complete(values) => Ok(values),
            BatchReadResponse::Partial(slots) => self.fill_missing(ids, slots).await,
        }
    }

    async fn fill_missing(&mut self, ids: &[VirtSfr], slots: Vec<Option<u32>>) -> Result<Vec<u32>> {
        let mut values = Vec::with_capacity(ids.len());
        for (id, slot) in ids.iter().zip(slots) {
            values.push(match slot {
                Some(value) => value,
                None => self.read_vsfr_u32(*id).await?,
            });
        }
        Ok(values)
    }

    async fn read_each(&mut self, ids: &[VirtSfr]) -> Result<Vec<u32>> {
        let mut values = Vec::with_capacity(ids.len());
        for id in ids {
            values.push(self.read_vsfr_u32(*id).await?);
        }
        Ok(values)
    }
}

fn falls_back_to_single_reads(error: &Error) -> bool {
    !matches!(error, Error::VsfrBatchEmpty) && !error.is_connection_lost()
}
