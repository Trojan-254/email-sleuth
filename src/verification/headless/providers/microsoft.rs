//! Microsoft/Outlook specific email verification implementation.

use crate::core::error::Result;
use crate::core::models::FoundEmailData;
use crate::verification::headless::utils::browser;
use crate::verification::headless::utils::selectors::MicrosoftSelectors;
use fantoccini::{Client, ClientBuilder};
use std::time::{Duration, Instant};
use tracing;

/// Checks Hotmail/Outlook/Live.com email existence using the password recovery flow via headless browser.
///
/// # Arguments
/// * `config` - Application configuration
/// * `email` - The email address to verify
/// * `webdriver_url` - URL of the running WebDriver instance
///
/// # Returns
/// * `Result<Option<FoundEmailData>>` - Verification result or error
pub async fn check_hotmail_headless(
    email: &str,
    webdriver_url: &str,
) -> Result<Option<FoundEmailData>> {
    let task_label = format!("[Hotmail Headless: {}]", email);
    tracing::info!(target: "verification_headless", "{} Starting check via {}", task_label, webdriver_url);
    let start_time = Instant::now();

    // Create WebDriver client
    let client = match create_client(webdriver_url).await {
        Ok(c) => c,
        Err(e) => {
            tracing::error!(target: "verification_headless", "{} Critical failure: Could not create WebDriver client: {}", task_label, e);
            return Err(e);
        }
    };

    let result = perform_microsoft_verification(&client, email, &task_label).await;

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

/// Creates a WebDriver client for Microsoft verification.
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

    let mut builder = ClientBuilder::native();

    builder.capabilities(caps);

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

/// Performs the Microsoft/Outlook verification process.
async fn perform_microsoft_verification(
    client: &Client,
    email: &str,
    task_label: &str,
) -> Result<Option<FoundEmailData>> {
    let page_load_timeout = Duration::from_secs(25);
    let element_wait_timeout = Duration::from_secs(15);

    tracing::debug!(target: "verification_headless", "{} Navigating to Microsoft password reset page...", task_label);
    browser::navigate_to(
        client,
        "https://account.live.com/password/reset",
        MicrosoftSelectors::email_input(),
        page_load_timeout,
        task_label,
    )
    .await?;

    browser::wait_and_type(
        client,
        MicrosoftSelectors::email_input(),
        email,
        element_wait_timeout,
        task_label,
    )
    .await?;

    browser::wait_and_click(
        client,
        MicrosoftSelectors::submit_button(),
        element_wait_timeout,
        task_label,
    )
    .await?;

    tracing::debug!(target: "verification_headless", "{} Checking for CAPTCHA...", task_label);
    let captcha_check_result = client
        .wait()
        .at_most(element_wait_timeout)
        .for_element(MicrosoftSelectors::captcha())
        .await;

    if captcha_check_result.is_ok() {
        tracing::warn!(target: "verification_headless", 
            "{} Verification inconclusive due to CAPTCHA", task_label);
        return Ok(None);
    }

    tracing::debug!(target: "verification_headless", "{} Checking for outcome indicators...", task_label);

    let outcome_checks = vec![
        // Email exists indicators
        (MicrosoftSelectors::exists_verify_identity(), true),
        (MicrosoftSelectors::exists_authenticator(), true),
        (MicrosoftSelectors::not_exists_error1(), false),
        (MicrosoftSelectors::not_exists_error2(), false),
    ];

    // Check outcomes
    match browser::check_outcomes(client, outcome_checks, element_wait_timeout, task_label).await? {
        Some(exists) => {
            if exists {
                tracing::info!(target: "verification_headless", 
                    "{} Determined account LIKELY EXISTS (Verification options/code entry found).", task_label);
                Ok(Some(FoundEmailData {
                    email: email.to_string(),
                    confidence: 7,
                    source: "headless_hotmail".to_string(),
                    is_generic: false,
                    verification_status: Some(true),
                    verification_message:
                        "Verified via Microsoft password recovery flow (options/code shown)"
                            .to_string(),
                }))
            } else {
                tracing::info!(target: "verification_headless", 
                    "{} Determined account LIKELY DOES NOT EXIST (Error message found).", task_label);
                Ok(Some(FoundEmailData {
                    email: email.to_string(),
                    confidence: 0,
                    source: "headless_hotmail".to_string(),
                    is_generic: false,
                    verification_status: Some(false),
                    verification_message:
                        "Non-existent per Microsoft password recovery flow (error shown)"
                            .to_string(),
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
