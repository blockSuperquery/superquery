//! Minimal JSON-RPC client used to probe chain connectivity.
//!
//! This is *not* the full chain abstraction — the `BlockchainService` trait and
//! typed clients (`subxt`/`alloy`) arrive at M3. This is deliberately a thin,
//! chain-family-agnostic height probe so GATE 1 can confirm we can talk to a real
//! node before any indexing logic is built on top.

use serde_json::json;
use thiserror::Error;

/// Errors from the JSON-RPC probe.
#[derive(Debug, Error)]
pub enum RpcError {
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("rpc returned an error: {0}")]
    Rpc(String),
    #[error("unexpected rpc response shape: {0}")]
    Shape(String),
}

/// Which RPC dialect to speak when probing height.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChainFamily {
    /// Substrate: `chain_getHeader` → `.number` (hex).
    Substrate,
    /// EVM: `eth_blockNumber` → hex.
    Evm,
}

/// A thin JSON-RPC 2.0 client over HTTP.
pub struct JsonRpcClient {
    endpoint: String,
    http: reqwest::Client,
}

impl JsonRpcClient {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            http: reqwest::Client::new(),
        }
    }

    /// Fetch the latest block height, auto-detecting nothing — the caller states
    /// the chain family. Returns the decoded height.
    pub async fn latest_height(&self, family: ChainFamily) -> Result<u64, RpcError> {
        match family {
            ChainFamily::Evm => {
                let hex = self.call_string("eth_blockNumber", json!([])).await?;
                parse_hex_u64(&hex)
            }
            ChainFamily::Substrate => {
                let header = self.call("chain_getHeader", json!([])).await?;
                let num = header
                    .get("number")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| RpcError::Shape("missing header.number".into()))?;
                parse_hex_u64(num)
            }
        }
    }

    async fn call(&self, method: &str, params: serde_json::Value) -> Result<serde_json::Value, RpcError> {
        let body = json!({"jsonrpc": "2.0", "id": 1, "method": method, "params": params});
        let resp: serde_json::Value = self
            .http
            .post(&self.endpoint)
            .json(&body)
            .send()
            .await?
            .json()
            .await?;
        if let Some(err) = resp.get("error") {
            return Err(RpcError::Rpc(err.to_string()));
        }
        resp.get("result")
            .cloned()
            .ok_or_else(|| RpcError::Shape("missing result".into()))
    }

    async fn call_string(&self, method: &str, params: serde_json::Value) -> Result<String, RpcError> {
        let v = self.call(method, params).await?;
        v.as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| RpcError::Shape(format!("expected string result, got {v}")))
    }
}

/// Parse a `0x`-prefixed hex quantity into `u64`.
fn parse_hex_u64(s: &str) -> Result<u64, RpcError> {
    let trimmed = s.strip_prefix("0x").unwrap_or(s);
    u64::from_str_radix(trimmed, 16).map_err(|e| RpcError::Shape(format!("bad hex '{s}': {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_parsing() {
        assert_eq!(parse_hex_u64("0x0").unwrap(), 0);
        assert_eq!(parse_hex_u64("0x10").unwrap(), 16);
        assert_eq!(parse_hex_u64("ff").unwrap(), 255);
        assert!(parse_hex_u64("0xzz").is_err());
    }
}
