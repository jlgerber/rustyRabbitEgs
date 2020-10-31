//! # Rpc - RabbitMQ Example in Rust
//! 
//! This crate implements the RPC example #6 from the RabbitMQ Tutorials
//! - [RabbitMQ RPC Tutorial](https://www.rabbitmq.com/tutorials/tutorial-six-python.html)
//! 
//! The tutorial illustrates a pattern that solves the need to run a function 
//! on a remote computer and wait for the result. 
//!
//! The premise is simple. Create a second, exclusive callback queue per client, and 
//! provide it's name via the `reply_to` property when publishing the request, along
//! with a unique correlation id (`correlation_id` property) generated once per request.
//! The correlation id will be generated with the Uuid crate, guaranteeing uniqueness.
//! 
//! We will create two structs - a Client struct and a Server struct. The Client will
//! be responsible for setting up the reply_to queue, along with generating the 
//! correlation id, and for making the request to the server, and processing the results.
//!
//! The Server will be responsible for basic setup as well as processing the incoming 
//! requests, calculating the fibonacci results, and sending them back based on the appropriate
//! queue as dictated by the `reply_to` property. It will also set the `correlation_id` 
//! from the incoming message property.
//!
//! Because Lapin is an async api, this will be slightly trickier than if it were sync.
//! One of the interesting challenges is that the Client code which processes the results 
//! must do so in a separate tag - potentially even a separate thread - depending upon the 
//! executor. This means any data that needs to be returned to the calling scope will need
//! to be wrapped in `an Arc<Mutex>`. 
//!
//! Also, while Lapin has its own executor for the framework, any async work that needs to be 
//! done will require the use of an executor from one of the many crates available. I chose
//! to use async_std. However, there are numerous other executors that I could have used.
//! Tokio, smol, etc etc
//!
//! ## Interesting Crates
//! In addition to `Lapin`, there are a number of other crates used herein. 
//! - tracing: Used to handle logging duties, `tracing` has a reputation for making 
//! tracing logs through an async ecosystem easier. i have no complaints.
//! - uuid: Used to generate the uuid string. Straightforward.
//! - structopt: My goto wrapper around clap making it really trivial to generate clis.
//! - strum: THis crate provides a number of procedural macros which make dealing with 
//! simple enums simple. I use it to convert back and forth between strings and variants. Its 
//! great for helping to provide more typesafety to an api which would otherwise be stringly
//! typed (due to laziness mostly)
//! 
use async_std::task;
use lapin::{
    Channel,
    Connection,
    ConnectionProperties, 
    Result as AsyncResult
};
use std::env;
use std::iter::Iterator;
use tracing::{info};

pub mod fib;
pub use fib::fib;

pub mod constants;
pub use constants::*;

pub mod log_level;
pub use log_level::LogLevel;

pub mod rpc_client;
pub use rpc_client::FibClient;

pub mod rpc_server;
pub use rpc_server::FibRpcServer;

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
