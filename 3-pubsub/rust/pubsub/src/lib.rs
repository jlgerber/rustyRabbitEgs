use lapin::ExchangeKind;

pub const LOCALHOST: &'static str = "amqp://127.0.0.1:5672/%2f";
pub const EXCHANGE: &'static str = "logs_rust";
pub const EXCHANGE_TYPE: ExchangeKind = ExchangeKind::Fanout;
pub const ROUTING_KEY: &'static str = ""; // empty
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