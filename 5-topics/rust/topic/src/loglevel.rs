use strum::{EnumString, AsRefStr};

/// A standard log level
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, EnumString, AsRefStr)]
pub enum LogLevel{
    #[strum(serialize="debug")]
    Debug,
    #[strum(serialize="info")]
    Info,
    #[strum(serialize="warn", serialize="warning")]
    Warn,
    #[strum(serialize="error", serialize="err")]
    Error
}
