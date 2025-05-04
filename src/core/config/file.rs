//! Defines the structure mirroring the TOML configuration file format.

use serde::Deserialize;

#[derive(Deserialize, Debug, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct ConfigFile {
    #[serde(default)]
    pub(crate) network: NetworkConfig,
    #[serde(default)]
    pub(crate) dns: DnsConfig,
    #[serde(default)]
    pub(crate) smtp: SmtpConfig,
    #[serde(default)]
    pub(crate) scraping: ScrapingConfig,
    #[serde(default)]
    pub(crate) verification: VerificationConfig,
    #[serde(default)]
    pub(crate) advanced_verification: AdvancedVerificationConfig,
}

#[derive(Deserialize, Debug, Default, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct NetworkConfig {
    pub(crate) request_timeout: Option<u64>,
    pub(crate) min_sleep: Option<f32>,
    pub(crate) max_sleep: Option<f32>,
    pub(crate) user_agent: Option<String>,
}

#[derive(Deserialize, Debug, Default, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct DnsConfig {
    pub(crate) dns_timeout: Option<u64>,
    pub(crate) dns_servers: Option<Vec<String>>,
}

#[derive(Deserialize, Debug, Default, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct SmtpConfig {
    pub(crate) smtp_timeout: Option<u64>,
    pub(crate) smtp_sender_email: Option<String>,
    pub(crate) max_verification_attempts: Option<u32>,
}

#[derive(Deserialize, Debug, Default, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct ScrapingConfig {
    pub(crate) common_pages: Option<Vec<String>>,
    pub(crate) generic_email_prefixes: Option<Vec<String>>,
}

#[derive(Deserialize, Debug, Default, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct VerificationConfig {
    pub(crate) confidence_threshold: Option<u8>,
    pub(crate) generic_confidence_threshold: Option<u8>,
    pub(crate) max_alternatives: Option<usize>,
    pub(crate) max_concurrency: Option<usize>,
    pub(crate) early_termination_threshold: Option<u8>,
}

#[derive(Deserialize, Debug, Default, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct AdvancedVerificationConfig {
    pub(crate) enable_api_checks: Option<bool>,
    pub(crate) enable_headless_checks: Option<bool>,
    pub(crate) webdriver_url: Option<String>,
    pub(crate) chromedriver_path: Option<String>,
}
