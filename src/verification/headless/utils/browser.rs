//! Browser interaction utilities for headless verification workflows.

use crate::core::error::{AppError, Result};
use fantoccini::{Client, Locator};
use futures::future::select_ok;
use std::pin::Pin;
use std::time::Duration;
use tokio::time::sleep;
use tracing;

/// Waits for an element to be clickable, then clicks it.
///
/// # Arguments
/// * `client` - The WebDriver client
/// * `locator` - The element locator
/// * `timeout` - Maximum time to wait for the element
/// * `label` - Task label for logging
pub async fn wait_and_click(
    client: &Client,
    locator: Locator<'_>,
    timeout: Duration,
    label: &str,
) -> Result<()> {
    tracing::debug!(target: "verification_headless", "{} Waiting for element to click: {:?}", label, locator);

    let element = client.wait().at_most(timeout).for_element(locator).await?;

    tracing::debug!(target: "verification_headless", "{} Found element, clicking...", label);
    element.click().await?;

    // Brief pause to allow page to respond
    sleep(Duration::from_millis(300)).await;
    Ok(())
}

/// Waits for an input element, then enters text.
///
/// # Arguments
/// * `client` - The WebDriver client
/// * `locator` - The input element locator
/// * `text` - Text to enter
/// * `timeout` - Maximum time to wait for the element
/// * `label` - Task label for logging
pub async fn wait_and_type(
    client: &Client,
    locator: Locator<'_>,
    text: &str,
    timeout: Duration,
    label: &str,
) -> Result<()> {
    tracing::debug!(target: "verification_headless", "{} Waiting for input element: {:?}", label, locator);

    let input = client.wait().at_most(timeout).for_element(locator).await?;

    tracing::debug!(target: "verification_headless", "{} Found input, typing: {}", label, text);
    input.send_keys(text).await?;

    // Brief pause to allow input to complete
    sleep(Duration::from_millis(300)).await;
    Ok(())
}

/// Navigates to a URL and waits for page to load.
///
/// # Arguments
/// * `client` - The WebDriver client
/// * `url` - The URL to navigate to
/// * `ready_locator` - Locator for element indicating page is ready
/// * `timeout` - Maximum time to wait for page to load
/// * `label` - Task label for logging
pub async fn navigate_to(
    client: &Client,
    url: &str,
    ready_locator: Locator<'_>,
    timeout: Duration,
    label: &str,
) -> Result<()> {
    tracing::debug!(target: "verification_headless", "{} Navigating to: {}", label, url);

    client.goto(url).await.map_err(|e| {
        tracing::error!("{} Failed to navigate: {}", label, e);
        AppError::from(e)
    })?;

    tracing::debug!(target: "verification_headless", "{} Waiting for page to load...", label);
    client
        .wait()
        .at_most(timeout)
        .for_element(ready_locator)
        .await?;

    tracing::debug!(target: "verification_headless", "{} Page loaded successfully", label);

    sleep(Duration::from_millis(300)).await;
    Ok(())
}

/// Checks for multiple outcome indicators concurrently with a timeout.
///
/// # Arguments
/// * `client` - The WebDriver client
/// * `outcome_checks` - Vec of pairs (locator, outcome value if found)
/// * `timeout` - Maximum time to wait for any outcome
/// * `label` - Task label for logging
pub async fn check_outcomes<T: Clone + std::fmt::Debug + Send + 'static>(
    client: &Client,
    outcome_checks: Vec<(Locator<'_>, T)>,
    timeout: Duration,
    label: &str,
) -> Result<Option<T>> {
    tracing::debug!(target: "verification_headless", "{} Checking for {} possible outcomes...", label, outcome_checks.len());

    let mut outcome_futures: Vec<Pin<Box<dyn futures::Future<Output = Result<T>> + Send>>> =
        Vec::new();

    for (locator, outcome) in outcome_checks {
        let outcome_clone = outcome.clone();
        let client_ref = client;
        let future = Box::pin(async move {
            match client_ref
                .wait()
                .at_most(timeout)
                .for_element(locator)
                .await
            {
                Ok(_) => Ok(outcome_clone),
                Err(e) => Err(AppError::from(e)),
            }
        });
        outcome_futures.push(future);
    }

    match select_ok(outcome_futures).await {
        Ok((outcome, _)) => {
            tracing::info!(target: "verification_headless", "{} Found outcome: {:?}", label, outcome);
            Ok(Some(outcome))
        }
        Err(e) => {
            tracing::warn!(target: "verification_headless", "{} No outcomes detected within timeout: {}", label, e);
            Ok(None)
        }
    }
}
