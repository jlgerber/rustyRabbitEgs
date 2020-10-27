//! # pubsub example
//! 
//! This is a port of the python example #3 int the tutorial
use ::anyhow::anyhow;
use ::anyhow::Error as AnyhowError;
use lapin::ExchangeKind;
//use lazy_static::lazy_static;
use std::str::FromStr;
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


use std::fmt;
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


// /// validate that the provided key is valid. To be a valid topic key, the LocLogLeve
// pub fn topic_key_is_valid(key: &LocLogLevel) -> bool {
//     // lazy_static! {
//     //     static ref RE: Regex = Regex::new(r"^(([a-zA-Z0-9]\.)*[a-zA-Z0-9])+\.(([a-zA-Z0-9]\.)*[a-zA-Z0-9])+$").unwrap();
//     // } 
//     // RE.is_match(&key.to_string())
//     key.is_specific()
// }


pub const LOCALHOST: &'static str = "amqp://127.0.0.1:5672/%2f";
pub const EXCHANGE: &'static str = "routed_logs_rust";
pub const EXCHANGE_TYPE: ExchangeKind = ExchangeKind::Topic;
// we are going to create the queue and delete it when finished. (declaring it exlusive and supplying a blank name)
pub const QUEUE: &'static str = "";

pub mod quit_service {
    use std::io;
    pub fn prompt() {
        println!("Type q or exit to quit");
        loop {
            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
               Ok(_goes_into_input_above) => {},
               Err(_no_updates_is_fine) => {},
            }
            let input = input.trim().to_string();
            if input == "q" || input == "quit" || input == "exit" {
               return 
           }
       }
    }
}