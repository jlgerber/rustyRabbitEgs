//! # pubsub example
//! 
//! This is a port of the python example #3 int the tutorial
use lapin::ExchangeKind;

pub mod loglevel;
pub use loglevel::LogLevel;
pub mod location;
pub use location::Location;
pub mod loc_loglevel;
pub use loc_loglevel::LocLogLevel;

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