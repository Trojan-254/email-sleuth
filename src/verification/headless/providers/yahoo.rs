//! Yahoo Mail specific email verification implementation.

use crate::core::error::Result;
use crate::core::models::FoundEmailData;
use crate::verification::headless::utils::browser;
use crate::verification::headless::utils::selectors::YahooSelectors;

use fantoccini::Client;
use std::time::{Duration, Instant};
use tracing;

/// Checks Yahoo email existence using the password recovery flow via headless browser.
///
/// # Arguments
/// * `config` - Application configuration
/// * `email` - The email address to verify
/// * `webdriver_url` - URL of the running WebDriver instance
///
/// # Returns
/// * `Result<Option<FoundEmailData>>` - Verification result or error
pub async fn check_yahoo_headless(
    email: &str,
    webdriver_url: &str,
) -> Result<Option<FoundEmailData>> {
    let task_label = format!("[Yahoo Headless: {}]", email);
    tracing::info!(target: "verification_headless", "{} Starting check via {}", task_label, webdriver_url);
    let start_time = Instant::now();

    let client = match create_client(webdriver_url).await {
        Ok(c) => c,
        Err(e) => {
            tracing::error!(target: "verification_headless", "{} Critical failure: Could not create WebDriver client: {}", task_label, e);
            return Err(e);
        }
    };

    let result = perform_yahoo_verification(&client, email, &task_label).await;

    if let Err(e) = client.close().await {
        tracing::warn!(target: "verification_headless", "{} Failed to close WebDriver client cleanly: {}", task_label, e);
    }

    let duration = start_time.elapsed();
    match &result {
        Ok(Some(_)) => {
            tracing::info!(target: "verification_headless", "{} Check finished in {:.2?}. Result: Conclusive", task_label, duration);
        }
        Ok(None) => {
            tracing::info!(target: "verification_headless", "{} Check finished in {:.2?}. Result: Inconclusive", task_label, duration);
        }
        Err(e) => {
            tracing::error!(target: "verification_headless", "{} Check failed in {:.2?}: {}", task_label, duration, e);
        }
    }

    result
}

/// Creates a WebDriver client for Yahoo verification.
async fn create_client(webdriver_url: &str) -> Result<Client> {
    let mut caps = serde_json::map::Map::new();
    let mut chrome_opts = serde_json::map::Map::new();

    let args = vec![
        "--headless=new",
        "--no-sandbox",
        "--disable-gpu",
        "--disable-dev-shm-usage",
        "--window-size=1024,768",
        "--disable-extensions",
        "--disable-background-networking",
        "--disable-sync",
        "--disable-translate",
        "--mute-audio",
        "--safebrowsing-disable-auto-update",
        "--ignore-certificate-errors",
        "--log-level=1",
    ];
    chrome_opts.insert("args".to_string(), serde_json::json!(args));
    caps.insert("browserName".to_string(), serde_json::json!("chrome"));
    caps.insert(
        "goog:chromeOptions".to_string(),
        serde_json::json!(chrome_opts),
    );

    tracing::debug!(target: "verification_headless", "Connecting to WebDriver at {}...", webdriver_url);
    let mut builder = fantoccini::ClientBuilder::native();
    let builder = builder.capabilities(caps);
    match builder.connect(webdriver_url).await {
        Ok(client) => {
            tracing::info!(target: "verification_headless", "WebDriver client connected successfully.");
            Ok(client)
        }
        Err(e) => {
            tracing::error!(target: "verification_headless", "Failed to connect to WebDriver at {}: {}", webdriver_url, e);
            Err(e.into())
        }
    }
}

/// Performs the Yahoo verification process.
async fn perform_yahoo_verification(
    client: &Client,
    email: &str,
    task_label: &str,
) -> Result<Option<FoundEmailData>> {
    let page_load_timeout = Duration::from_secs(20);
    let element_wait_timeout = Duration::from_secs(15);

    tracing::debug!(target: "verification_headless", "{} Navigating to Yahoo password reset page...", task_label);
    browser::navigate_to(
        client,
        "https://login.yahoo.com/forgot",
        YahooSelectors::email_input(),
        page_load_timeout,
        task_label,
    )
    .await?;

    browser::wait_and_type(
        client,
        YahooSelectors::email_input(),
        email,
        element_wait_timeout,
        task_label,
    )
    .await?;

    browser::wait_and_click(
        client,
        YahooSelectors::submit_button(),
        element_wait_timeout,
        task_label,
    )
    .await?;

    tracing::debug!(target: "verification_headless", "{} Checking for outcome indicators...", task_label);

    let outcome_checks = vec![
        (YahooSelectors::exists_recaptcha(), true),
        (YahooSelectors::exists_verification_code(), true),
        (YahooSelectors::exists_challenge_selector(), true),
        (YahooSelectors::not_exists_error(), false),
        (YahooSelectors::account_disabled(), false),
    ];

    match browser::check_outcomes(client, outcome_checks, element_wait_timeout, task_label).await? {
        Some(exists) => {
            if exists {
                tracing::info!(target: "verification_headless", 
                    "{} Determined account LIKELY EXISTS (Verification/Captcha/Options found).", task_label);
                Ok(Some(FoundEmailData {
                    email: email.to_string(),
                    confidence: 8,
                    source: "headless_yahoo".to_string(),
                    is_generic: false,
                    verification_status: Some(true),
                    verification_message:
                        "Verified via Yahoo password recovery flow (options/code/captcha shown)"
                            .to_string(),
                }))
            } else {
                tracing::info!(target: "verification_headless", 
                    "{} Determined account LIKELY DOES NOT EXIST or IS DISABLED (Error message/Locked found).", task_label);
                Ok(Some(FoundEmailData {
                    email: email.to_string(),
                    confidence: 0,
                    source: "headless_yahoo".to_string(),
                    is_generic: false,
                    verification_status: Some(false),
                    verification_message:
                        "Non-existent or disabled per Yahoo password recovery flow".to_string(),
                }))
            }
        }
        None => {
            tracing::warn!(target: "verification_headless", 
                "{} Could not determine outcome (all indicators timed out).", task_label);
            Ok(None)
        }
    }
}
