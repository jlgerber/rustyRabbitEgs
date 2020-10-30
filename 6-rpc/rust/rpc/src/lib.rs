use async_std::task;
use lapin::{
     Connection,Channel,
    ConnectionProperties, Result as AsyncResult
};
use std::env;
use std::iter::Iterator;
use tracing::{info};

pub mod fib;
pub use fib::ffib;

pub mod constants;
pub use constants::*;

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

#[derive(Debug)]
pub struct SimpleClient {
    pub conn: Connection,
    pub chan: Channel,
}

impl SimpleClient {
    pub fn new() -> AsyncResult<Self> {

        task::block_on(async {
            let addr = env::var("AMQP_ADDR").unwrap_or_else(|_| LOCALHOST.into());
            
            let conn = Connection::connect(
                &addr,
                ConnectionProperties::default(),
            )
            .await?;

            info!("established connection to Rabbit server via {}", &addr);

            let chan = conn.create_channel().await?;
            info!("created channel");

        
        Ok(Self {
                conn,
                chan,
            })
        })
    }
}

pub mod rpc_client;
pub use rpc_client::FibClient;
pub mod rpc_server;
pub use rpc_server::FibRpcServer;