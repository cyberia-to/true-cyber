use crate::{Result, error};
use cyb_core::robot::remote::{self, Transport};
use serde_json::Value;
use std::{io::Read, time::Duration};
pub struct Http {
    base: String,
    pub network: [u8; 32],
    agent: ureq::Agent,
}
impl Http {
    pub fn new(base: &str, network: [u8; 32]) -> Result<Self> {
        if base.len() > 4096
            || !(base.starts_with("http://") || base.starts_with("https://"))
            || base.contains(['?', '#', '@'])
        {
            return Err(
                "endpoint must be an http(s) URL without credentials/query/fragment".into(),
            );
        }
        Ok(Self {
            base: base.trim_end_matches('/').into(),
            network,
            agent: ureq::AgentBuilder::new()
                .timeout(Duration::from_secs(30))
                .redirects(0)
                .build(),
        })
    }
    pub fn verify(&mut self) -> Result<()> {
        remote::verify_capabilities(&self.capabilities()?, self.network)
    }
    pub fn history(&mut self, after: Option<u64>) -> Result<Value> {
        self.verify()?;
        let query = after
            .map(|n| format!("?after={n}&limit=16"))
            .unwrap_or("?limit=16".into());
        value(
            self.agent
                .get(&format!("{}/v3/history{query}", self.base))
                .call()
                .map_err(error)?,
            32 * 1024 * 1024,
        )
    }
}
fn value(response: ureq::Response, max: usize) -> Result<Value> {
    if !(200..300).contains(&response.status()) {
        return Err(format!("HTTP {}", response.status()));
    }
    let mut bytes = Vec::new();
    response
        .into_reader()
        .take(max as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(error)?;
    if bytes.len() > max {
        return Err("HTTP response exceeds bound".into());
    }
    serde_json::from_slice(&bytes).map_err(error)
}
impl Transport for Http {
    fn capabilities(&mut self) -> Result<Value> {
        value(
            self.agent
                .get(&format!("{}/capabilities", self.base))
                .call()
                .map_err(error)?,
            1024 * 1024,
        )
    }
    fn lookup(&mut self, path: &str) -> Result<Option<Value>> {
        match self.agent.get(&format!("{}{path}", self.base)).call() {
            Ok(r) => value(r, 1024 * 1024).map(Some),
            Err(ureq::Error::Status(404, _)) => Ok(None),
            Err(e) => Err(error(e)),
        }
    }
    fn submit(&mut self, request: [u8; 32], bytes: &[u8]) -> Result<Value> {
        value(
            self.agent
                .post(&format!("{}/v3/action", self.base))
                .set("Content-Type", "application/json")
                .set("Idempotency-Key", &remote::hex(&request))
                .send_bytes(bytes)
                .map_err(|e| format!("native result unresolved: {e}; original request retained"))?,
            1024 * 1024,
        )
    }
}
