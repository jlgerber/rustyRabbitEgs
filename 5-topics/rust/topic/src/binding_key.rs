//! binding_key
//!
//! # BindingKey
//! struct which represents a BindingKey
//! 
use ::anyhow::anyhow;
use anyhow::Error as AnyhowError;
use std::fmt;
use std::str::FromStr;

use crate::LogLevel;
use crate::Location;
use std::convert::From;
use crate::RoutingKey;

/// a location aware log level, specified as a dot delimited
/// <location>.<level>
/// Either location of level or both may be specified as "*" to 
/// indicate any
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BindingKey {
    /// BindingKey
    Pair{
    location: Location,
    level: LogLevel
    },
    /// Any Level but specific Location
    AnyLevel(Location),
    /// Any Location but specific LogLevel
    AnyLoc(LogLevel),
    /// Any Location and LogLevel
    Any,
}

impl BindingKey {
    /// Does the variant of BindingKey represent a distinct pair of Location and Level?
    pub fn is_specific(&self) -> bool {
        if let Self::Pair{..} = self {true} else {false} 
    }
}

impl From<RoutingKey> for BindingKey {
    fn from(k: RoutingKey) -> Self {
        let RoutingKey{location, level} = k;
        BindingKey::Pair{location, level}
    }
}

impl FromStr for BindingKey {
    type Err = AnyhowError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let pieces = s.split(".").collect::<Vec<_>>();
        if pieces.len() > 2 {
            return Err(anyhow!("malformed input: {}", s));
        }
        if pieces.len() <2 {
            if pieces[0] == "#" {
                return Ok(Self::Any);
            } 
            return Err(anyhow!("malformed input: {}", s));
        }
        if pieces[0] == "*" {
            if pieces[1] == "*" {
                return Ok(Self::Any)
            }
            let level = LogLevel::from_str(pieces[1]).map_err(|_| anyhow!("malfomed level: {}", pieces[1]) )?;
     
            return Ok(Self::AnyLoc(level));
        }

        let location = Location::from_str(pieces[0]).map_err(|_| anyhow!("malformed location: {}", pieces[0]))?;
        if pieces[1] == "*" {
            return Ok(Self::AnyLevel(location));
        }

        let level = LogLevel::from_str(pieces[1]).map_err(|_| anyhow!("malfomed level: {}", pieces[1]) )?;
        return Ok(Self::Pair{location, level})
    }
}


impl fmt::Display for BindingKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pair{location,level} => write!(f, "{}.{}",location.as_ref(), level.as_ref() ),
            Self::AnyLevel(loc) => write!(f,"{}.*", loc.as_ref()),
            Self::AnyLoc(level) => write!(f,"*.{}", level.as_ref()),
            Self::Any => write!(f, "#"),
        }
    }
}
