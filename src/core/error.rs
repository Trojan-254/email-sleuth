//! Defines the custom error types for the email-sleuth application.

use fantoccini::error::{CmdError, NewSessionError};
use std::{io, net::AddrParseError};
use thiserror::Error;
use url::ParseError as UrlParseError;

/// The primary error type for the email finding process.
#[derive(Error, Debug)]
pub enum AppError {
    /// Error occurring during configuration loading or validation.
    #[error("Configuration Error: {0}")]
    Config(String),

    /// Error initializing necessary components (e.g., clients, resolvers).
    #[error("Initialization Error: {0}")]
    Initialization(String),

    /// Error related to file input/output operations.
    #[error("IO Error: {0}")]
    Io(#[from] io::Error),

    /// Error during JSON serialization or deserialization.
    #[error("JSON Error: {0}")]
    Json(#[from] serde_json::Error),

    /// Error parsing a URL.
    #[error("URL Parsing Error: {0}")]
    UrlParse(#[from] UrlParseError),

    /// Error making HTTP requests via reqwest.
    #[error("HTTP Request Error: {0}")]
    Request(#[from] reqwest::Error),

    /// Error parsing HTML content.
    #[error("HTML Parsing Error: {0}")]
    HtmlParse(String),

    /// Error during DNS resolution.
    #[error("DNS Resolution Error: {0}")]
    Dns(#[from] trust_dns_resolver::error::ResolveError),

    /// Specific DNS error indicating the domain does not exist.
    #[error("Domain Not Found (NXDOMAIN): {0}")]
    NxDomain(String),

    /// Specific DNS error indicating no relevant records were found.
    #[error("No DNS Records Found (MX/A): {0}")]
    NoDnsRecords(String),

    /// DNS operation timed out.
    #[error("DNS Timeout for domain: {0}")]
    DnsTimeout(String),

    /// Error during SMTP communication setup or command execution.
    #[error("SMTP Error: {0}")]
    Smtp(#[from] lettre::transport::smtp::Error),

    /// Error related to SMTP command execution details.
    #[error("SMTP Command Error: Code={code}, Message='{message}'")]
    SmtpCommand {
        /// The SMTP status code.
        code: lettre::transport::smtp::response::Code,
        /// The message returned by the server.
        message: String,
    },

    /// Error specifically during the STARTTLS handshake.
    #[error("SMTP STARTTLS Error: {0}")]
    SmtpTls(String),

    /// Error parsing an IP address or socket address.
    #[error("Address Parsing Error: {0}")]
    AddrParse(#[from] AddrParseError),

    /// Error related to concurrency or task execution.
    #[error("Task Execution Error: {0}")]
    Task(String),

    /// An underlying error that doesn't fit other categories, using anyhow.
    #[error("Generic Error: {0}")]
    Generic(#[from] anyhow::Error),

    /// Indicates insufficient input data to proceed (e.g., missing name/website).
    #[error("Insufficient Input Data: {0}")]
    InsufficientInput(String),

    /// Failed to extract a domain from the provided URL.
    #[error("Failed to extract domain from URL: {0}")]
    DomainExtraction(String),

    /// SMTP verification returned a temporary failure (e.g., 4xx code).
    #[error("SMTP Temporary Failure: {0}")]
    SmtpTemporaryFailure(String),

    /// SMTP verification returned a permanent failure (e.g., 5xx code, user unknown).
    #[error("SMTP Permanent Failure: {0}")]
    SmtpPermanentFailure(String),

    /// SMTP verification was inconclusive (e.g., catch-all, timeout).
    #[error("SMTP Inconclusive: {0}")]
    SmtpInconclusive(String),

    /// Error connecting to the WebDriver instance.
    #[error("WebDriver Connection Error: {0}")]
    WebDriverConnection(String),

    /// Indicates that the verification process was actively blocked (e.g., by CAPTCHA).
    #[error("Verification Blocked: {0}")]
    VerificationBlocked(String),

    /// Error executing a command via WebDriver (Fantoccini).
    #[error("WebDriver Command Error: {0}")]
    FantocciniCmd(String),
}

// From implementations for Fantoccini errors
impl From<CmdError> for AppError {
    fn from(err: CmdError) -> Self {
        let msg = err.to_string();
        if msg.contains("element click intercepted") || msg.contains("element is not interactable")
        {
            // Potentially we can classify this as VerificationBlocked if it's consistently due to anti-bot measures?
            // AppError::VerificationBlocked(format!("Interaction blocked: {}", msg))
            AppError::FantocciniCmd(msg)
        } else {
            AppError::FantocciniCmd(msg)
        }
    }
}

impl From<NewSessionError> for AppError {
    fn from(err: NewSessionError) -> Self {
        AppError::WebDriverConnection(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
