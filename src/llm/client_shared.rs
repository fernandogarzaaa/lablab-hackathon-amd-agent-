//! Shared HTTP client and retry logic for all LLM providers.
//!
//! A single connection-pooled reqwest::Client is constructed once and cloned
//! to all providers. Retry logic with exponential backoff handles transient
//! failures (5xx, 429, network errors).

use anyhow::Result;
use reqwest::Client;
use std::time::Duration;
use tracing::warn;

/// Configuration for the shared HTTP client.
#[derive(Debug, Clone)]
pub struct HttpClientConfig {
    pub connect_timeout: Duration,
    pub request_timeout: Duration,
    pub max_retries: u32,
    pub initial_backoff_ms: u64,
    pub max_backoff_ms: u64,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(10),
            request_timeout: Duration::from_secs(120),
            max_retries: 5,
            initial_backoff_ms: 1_000,
            max_backoff_ms: 30_000,
        }
    }
}

/// A cloned reference to a single reqwest::Client with timeouts configured.
#[derive(Clone, Debug)]
pub struct SharedHttpClient {
    client: Client,
}

impl Default for SharedHttpClient {
    fn default() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl SharedHttpClient {
    pub fn new(config: &HttpClientConfig) -> Self {
        let client = Client::builder()
            .connect_timeout(config.connect_timeout)
            .timeout(config.request_timeout)
            .build()
            .expect("Failed to build HTTP client");
        Self { client }
    }

    pub fn inner(&self) -> &Client {
        &self.client
    }
}

/// Retry a fallible async operation with exponential backoff.
pub async fn retry_with_backoff<F, Fut, T>(config: &HttpClientConfig, mut operation: F) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut attempt = 0u32;
    let mut backoff_ms = config.initial_backoff_ms;

    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                attempt += 1;
                if attempt >= config.max_retries {
                    return Err(anyhow::anyhow!(
                        "Operation failed after {} retries: {}",
                        attempt,
                        e
                    ));
                }
                warn!(
                    "Attempt {} failed: {}, retrying in {}ms",
                    attempt, e, backoff_ms
                );
                tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                backoff_ms = (backoff_ms * 2).min(config.max_backoff_ms);
            }
        }
    }
}
