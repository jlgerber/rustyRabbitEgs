use strum::AsRefStr;
use strum::EnumString;


#[derive(Debug, PartialEq, AsRefStr, EnumString)]
pub enum LogLevel {
    #[strum(serialize = "debug")]
    Debug,
    #[strum(serialize = "info")]
    Info,
    #[strum(serialize = "warn")]
    Warn,
    #[strum(serialize = "error")]
    Error
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn loglevel_given_str_converts() {
        for (levelstr, loglevel) in &[
            ("debug",LogLevel::Debug),
            ("info", LogLevel::Info), 
            ("warn", LogLevel::Warn), 
            ("error", LogLevel::Error)
            ] {
            let level = LogLevel::from_str(levelstr);
            assert_eq!( level.unwrap(), *loglevel);
            }
        }
       

    #[test]
    fn loglevel_given_bad_str_fails() {
        let level = LogLevel::from_str("deffbug");
        assert!( level.is_err());
    }

    #[test]
    fn loglevel_to_string() {
        for (levelstr, loglevel) in &[
            ("debug",LogLevel::Debug),
            ("info", LogLevel::Info), 
            ("warn", LogLevel::Warn), 
            ("error", LogLevel::Error)
            ]
        {
            let level = loglevel.as_ref();
            assert_eq!( &level, levelstr);
        }
        
    }
}