use std::path::Path;
use std::time::Duration;

use crate::api_types::CommandReport;
use crate::dnsmasq::process;
use crate::error::AppResult;

#[derive(Clone, Debug)]
pub struct DnsmasqCommand {
    bin: String,
    timeout: Duration,
}

impl DnsmasqCommand {
    pub fn new(bin: impl Into<String>, timeout: Duration) -> Self {
        Self {
            bin: bin.into(),
            timeout,
        }
    }

    pub async fn test_config(&self, config_path: &Path) -> AppResult<CommandReport> {
        let arg = format!("--conf-file={}", config_path.display());
        process::run(&self.bin, &["--test", &arg], self.timeout).await
    }
}
