use std::str::FromStr;
use std::fmt;
use ::anyhow::anyhow;

use anyhow::Error as AnyhowError;

use crate::LogLevel;
use crate::Location;
/// a location aware log level, specified as a dot delimited
/// <location>.<level>
/// Either location of level or both may be specified as "*" to 
/// indicate any
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LocLogLevel {
    /// Specific Location and LogLevel
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

impl LocLogLevel {
    /// Does the variant of LocLogLevel represent a distinct pair of Location and Level?
    pub fn is_specific(&self) -> bool {
        if let Self::Pair{..} = self {true} else {false} 
    }
}

impl FromStr for LocLogLevel {
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


impl fmt::Display for LocLogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pair{location,level} => write!(f, "{}.{}",location.as_ref(), level.as_ref() ),
            Self::AnyLevel(loc) => write!(f,"{}.*", loc.as_ref()),
            Self::AnyLoc(level) => write!(f,"*.{}", level.as_ref()),
            Self::Any => write!(f, "#"),
        }
    }
}
