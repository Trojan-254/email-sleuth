mod client;
mod providers;
mod utils;
pub use providers::microsoft::check_hotmail_headless;
pub use providers::yahoo::check_yahoo_headless;
