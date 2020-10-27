//use async_amqp::*;
use lapin::{
    options::*, publisher_confirm::Confirmation, types::FieldTable, 
    BasicProperties, Connection,
    ConnectionProperties, Result, //message::DeliveryResult,
};
use tracing::info;
use std::env;

use work_queues::{LOCALHOST, QUEUE};

fn setup() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
    if env::args().len() < 2 {
        println!("Usage: client <message>");
        std::process::exit(1);
    }
    tracing_subscriber::fmt::init();
}


#[async_std::main]
async fn main() -> Result<()> {
    setup();

    let addr = std::env::var("AMQP_ADDR").unwrap_or_else(|_| LOCALHOST.into());

  
    let conn = Connection::connect(
        &addr,
        ConnectionProperties::default(),
    )
    .await?;
    info!("established connection to Rabbit server via {}", &addr);

    let channel_a = conn.create_channel().await?;
    info!("created channel");

    let queue = channel_a
        .queue_declare(
            QUEUE,
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;
    // what manner of vodo is this? what does `?queue` mean?
    // `tracing` uses the `?` prefix to indicate that a variable
    // should use std::debug to print. And `tracing` uses `%`
    // prefix to indicate that it should use std::display. Just
    // so you know....
    info!(?queue, "Declared queue 'hello'");

    let msg = env::args().skip(1).collect::<Vec<_>>().join(" ");
    
    info!("extracted message from environment");
    // basic_publish returns a PromiseChain<T>. 
    // which is a PinkySwear<Result<T>, Result<()>>
    // which is a `wtf`?
    // https://docs.rs/pinky-swear/5.0.1/pinky_swear/
    let confirm = channel_a
        .basic_publish(
            // exchange
            "", 
            // routing key
            QUEUE, 
            // options
            BasicPublishOptions::default(),
            // payload
            msg.as_bytes().to_vec(),
            // properties
            BasicProperties::default(),
        )
        .await?
        .await?; // two awaits to get from a doubly wrapped
        // Result 
    assert_eq!(confirm, Confirmation::NotRequested);
        
    Ok(())
    
}