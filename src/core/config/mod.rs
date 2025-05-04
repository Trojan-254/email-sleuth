//! Defines the core runtime `Config` struct, its defaults, and related utilities.
//! Submodules handle loading, building, and validation.

pub(crate) mod builder;
pub(crate) mod file;
pub(crate) mod loading;
pub(crate) mod validation;

pub use builder::ConfigBuilder;
pub use file::ConfigFile;

use crate::core::error::Result;
use regex::Regex;
use std::collections::HashSet;
use std::time::Duration;

/// Runtime configuration settings used by the email-sleuth core logic.
pub struct Config {
    pub request_timeout: Duration,
    pub sleep_between_requests: (f32, f32),
    pub user_agent: String,

    pub dns_timeout: Duration,
    pub dns_servers: Vec<String>,

    pub smtp_timeout: Duration,
    pub smtp_sender_email: String,
    pub max_verification_attempts: u32,

    pub common_pages_to_scrape: Vec<String>,
    pub email_regex: Regex,
    pub generic_email_prefixes: HashSet<String>,

    pub confidence_threshold: u8,
    pub generic_confidence_threshold: u8,
    pub max_alternatives: usize,
    pub max_concurrency: usize,

    pub enable_api_checks: bool,
    pub enable_headless_checks: bool,
    pub webdriver_url: Option<String>,
    pub chromedriver_path: Option<String>,

    pub early_termination_threshold: u8,

    pub loaded_config_path: Option<String>,
}

impl Config {
    fn build_default() -> Self {
        let common_pages = vec![
            "/contact",
            "/contact-us",
            "/contactus",
            "/contact_us",
            "/about",
            "/about-us",
            "/aboutus",
            "/about_us",
            "/team",
            "/our-team",
            "/our_team",
            "/meet-the-team",
            "/people",
            "/staff",
            "/company",
        ];
        let generic_prefixes: HashSet<String> = [
            "info",
            "contact",
            "hello",
            "help",
            "support",
            "admin",
            "office",
            "sales",
            "press",
            "media",
            "marketing",
            "jobs",
            "careers",
            "hiring",
            "privacy",
            "security",
            "legal",
            "membership",
            "team",
            "people",
            "general",
            "feedback",
            "enquiries",
            "inquiries",
            "mail",
            "email",
            "pitch",
            "invest",
            "investors",
            "ir",
            "webmaster",
            "newsletter",
            "apply",
            "partner",
            "partners",
            "ventures",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        let email_regex_pattern = r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b";
        let email_regex = Regex::new(email_regex_pattern)
            .expect("Default email regex pattern failed to compile. This is a bug.");
        let dns_servers = vec![
            "8.8.8.8".to_string(),
            "8.8.4.4".to_string(),
            "1.1.1.1".to_string(),
            "1.0.0.1".to_string(),
        ];

        Config {
            request_timeout: Duration::from_secs(10),
            sleep_between_requests: (0.1, 0.5),
            user_agent: format!("email-sleuth-core/{}", env!("CARGO_PKG_VERSION")),
            dns_timeout: Duration::from_secs(5),
            dns_servers,
            smtp_timeout: Duration::from_secs(5),
            smtp_sender_email: "verify-probe@example.com".to_string(),
            max_verification_attempts: 2,
            common_pages_to_scrape: common_pages.iter().map(|s| s.to_string()).collect(),
            email_regex,
            generic_email_prefixes: generic_prefixes,
            confidence_threshold: 4,
            generic_confidence_threshold: 7,
            max_alternatives: 5,
            max_concurrency: std::thread::available_parallelism()
                .map_or(1, |n| n.get())
                .max(1),
            enable_api_checks: false,
            enable_headless_checks: false,
            webdriver_url: None,
            chromedriver_path: None,
            early_termination_threshold: 9,
            loaded_config_path: None,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config::build_default()
    }
}

impl Clone for Config {
    fn clone(&self) -> Self {
        Self {
            request_timeout: self.request_timeout,
            sleep_between_requests: self.sleep_between_requests,
            user_agent: self.user_agent.clone(),
            dns_timeout: self.dns_timeout,
            dns_servers: self.dns_servers.clone(),
            smtp_timeout: self.smtp_timeout,
            smtp_sender_email: self.smtp_sender_email.clone(),
            max_verification_attempts: self.max_verification_attempts,
            common_pages_to_scrape: self.common_pages_to_scrape.clone(),
            email_regex: self.email_regex.clone(),
            generic_email_prefixes: self.generic_email_prefixes.clone(),
            confidence_threshold: self.confidence_threshold,
            generic_confidence_threshold: self.generic_confidence_threshold,
            max_alternatives: self.max_alternatives,
            max_concurrency: self.max_concurrency,
            enable_api_checks: self.enable_api_checks,
            enable_headless_checks: self.enable_headless_checks,
            webdriver_url: self.webdriver_url.clone(),
            chromedriver_path: self.chromedriver_path.clone(),
            early_termination_threshold: self.early_termination_threshold,
            loaded_config_path: self.loaded_config_path.clone(),
        }
    }
}

impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("request_timeout", &self.request_timeout)
            .field("sleep_between_requests", &self.sleep_between_requests)
            .field("user_agent", &self.user_agent)
            .field("dns_timeout", &self.dns_timeout)
            .field("dns_servers_count", &self.dns_servers.len())
            .field("smtp_timeout", &self.smtp_timeout)
            .field("smtp_sender_email", &self.smtp_sender_email)
            .field("max_verification_attempts", &self.max_verification_attempts)
            .field(
                "common_pages_to_scrape_count",
                &self.common_pages_to_scrape.len(),
            )
            .field("email_regex", &self.email_regex.as_str())
            .field(
                "generic_email_prefixes_count",
                &self.generic_email_prefixes.len(),
            )
            .field("confidence_threshold", &self.confidence_threshold)
            .field(
                "generic_confidence_threshold",
                &self.generic_confidence_threshold,
            )
            .field("max_alternatives", &self.max_alternatives)
            .field("max_concurrency", &self.max_concurrency)
            .field("enable_api_checks", &self.enable_api_checks)
            .field("enable_headless_checks", &self.enable_headless_checks)
            .field("webdriver_url", &self.webdriver_url)
            .field("chromedriver_path", &self.chromedriver_path)
            .field(
                "early_termination_threshold",
                &self.early_termination_threshold,
            )
            .field("loaded_config_path", &self.loaded_config_path)
            .finish()
    }
}

/// Utility function to get a random sleep duration based on [`Config`].
///
/// Uses the `sleep_between_requests` setting from the provided configuration.
pub fn get_random_sleep_duration(config: &Config) -> Duration {
    use rand::Rng;
    let (min, max) = config.sleep_between_requests;
    if min >= max {
        return Duration::from_secs_f32(min.max(0.0));
    }
    let duration_secs = rand::thread_rng().gen_range(min..max);
    Duration::from_secs_f32(duration_secs)
}
