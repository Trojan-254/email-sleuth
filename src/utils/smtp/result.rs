// src/utils/smtp/result.rs
//! Defines the result type for SMTP verification operations.

/// Represents the outcome of an SMTP verification attempt for an email address.
#[derive(Debug, Clone)]
pub struct SmtpVerificationResult {
    /// True = Exists, False = Does Not Exist, None = Inconclusive/Error
    pub exists: Option<bool>,
    /// Detailed message about the outcome.
    pub message: String,
    /// Suggests if retrying might yield a different result (e.g., for temporary errors).
    pub should_retry: bool,
    /// Indicates if the domain seems to accept all emails.
    pub is_catch_all: bool,
}
#[allow(dead_code)]
impl SmtpVerificationResult {
    /// Creates a conclusive result (email definitely exists or not).
    pub fn conclusive(exists: bool, message: String, is_catch_all: bool) -> Self {
        Self {
            exists: Some(exists),
            message,
            should_retry: false,
            is_catch_all,
        }
    }

    /// Creates an inconclusive result where retrying might help.
    pub fn inconclusive_retry(message: String) -> Self {
        Self {
            exists: None,
            message,
            should_retry: true,
            is_catch_all: false,
        }
    }

    /// Creates an inconclusive result where retrying is unlikely to help.
    pub fn inconclusive_no_retry(message: String) -> Self {
        Self {
            exists: None,
            message,
            should_retry: false,
            is_catch_all: false,
        }
    }

    /// Creates a result for a catch-all domain.
    pub fn catch_all(message: String) -> Self {
        Self {
            exists: None, // Inconclusive because we can't determine individual email validity
            message,
            should_retry: false, // No need to retry catch-all checks
            is_catch_all: true,
        }
    }
}
