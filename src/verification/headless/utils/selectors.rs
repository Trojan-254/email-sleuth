//! CSS selectors for different email providers' verification flows.

use fantoccini::Locator;

/// Yahoo Mail verification flow selectors.
pub struct YahooSelectors;

impl YahooSelectors {
    pub fn email_input() -> Locator<'static> {
        Locator::Css("#username")
    }

    pub fn submit_button() -> Locator<'static> {
        Locator::Css("button[name='verifyYid']")
    }

    pub fn exists_recaptcha() -> Locator<'static> {
        Locator::Css(".recaptcha-challenge")
    }

    pub fn exists_verification_code() -> Locator<'static> {
        Locator::Id("email-verify-challenge")
    }

    pub fn exists_challenge_selector() -> Locator<'static> {
        Locator::Id("challenge-selector-challenge")
    }

    pub fn not_exists_error() -> Locator<'static> {
        Locator::Css(".error-msg")
    }

    pub fn account_disabled() -> Locator<'static> {
        Locator::Css(".ctx-account_is_locked")
    }
}

/// Microsoft/Outlook verification flow selectors.
pub struct MicrosoftSelectors;

impl MicrosoftSelectors {
    pub fn email_input() -> Locator<'static> {
        Locator::Id("iSigninName")
    }

    pub fn submit_button() -> Locator<'static> {
        Locator::Id("resetPwdHipAction")
    }

    pub fn exists_verify_identity() -> Locator<'static> {
        Locator::Id("iSelectProofTitle")
    }

    pub fn exists_authenticator() -> Locator<'static> {
        Locator::Id("iEnterVerification")
    }

    pub fn not_exists_error1() -> Locator<'static> {
        Locator::Id("pMemberNameErr")
    }

    pub fn not_exists_error2() -> Locator<'static> {
        Locator::Id("iSigninNameError")
    }

    pub fn captcha() -> Locator<'static> {
        Locator::Css("#hipEnforcementContainer, iframe[src*='captcha'], iframe[title*='CAPTCHA']")
    }
}
