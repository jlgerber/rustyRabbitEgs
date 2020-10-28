//! a valid routing key in rabbit
use anyhow::anyhow;
use anyhow::Error as AnyhowError;
use crate::Location;
use crate::LogLevel;
use crate::BindingKey;

use std::fmt;
use std::str::FromStr;
use std::convert::TryFrom;

/// RoutingKey represents a valid routing key in our made up
/// example. 
#[derive(Debug, PartialEq, Eq)]
pub struct RoutingKey {
    pub location: Location,
    pub level: LogLevel
}

impl TryFrom<BindingKey> for RoutingKey {
    type Error = AnyhowError;

    fn try_from(key: BindingKey) -> Result<Self, Self::Error> {
        match key {
            BindingKey::Pair{location, level} => Ok(Self{location, level}),
            _ => Err(anyhow!("unable to convert {} to RoutingKey instance", &key.to_string()))
        }
    }
}

impl fmt::Display for RoutingKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.location.as_ref(), self.level.as_ref())
    }
}

impl FromStr for RoutingKey {
    type Err = AnyhowError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let pieces = s.split(".").collect::<Vec<_>>();
        if pieces.len() != 2 {
            return Err(anyhow!("cannot convert {} to RoutingKey. Input should be <location>.<loglevel>", s));
        }
        let location = Location::from_str(&pieces[0]).map_err(|e| anyhow!("{}", e))?;
        let level = LogLevel::from_str(&pieces[1]).map_err(|e| anyhow!("{}",e))?;
       Ok(RoutingKey{location, level})
    }
}
