use strum::{EnumString, AsRefStr};

/// A studio location
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, EnumString, AsRefStr)]
pub enum Location {
    #[strum(serialize="playa", serialize="dd")]
    Playa,
    #[strum(serialize="vancouver", serialize="bc")]
    Vancouver,
    #[strum(serialize="portland", serialize="pd")]
    Portland,
    #[strum(serialize="montreal", serialize="mt")]
    Montreal
}
