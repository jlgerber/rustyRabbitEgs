use anyhow::anyhow;
use anyhow::Result as AnyhowResult;
use lapin::{
    options::*, publisher_confirm::Confirmation, types::FieldTable, 
    BasicProperties, Connection, 
    ConnectionProperties, Result, 
};
use std::env;
use std::convert::AsRef;
use std::str::FromStr;
use tracing::{info,error};

use routing::{LOCALHOST, EXCHANGE, EXCHANGE_TYPE, Route};

// set up default logging level and initialize tracing
fn setup() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
    
    tracing_subscriber::fmt::init();
}

fn parse_args() -> AnyhowResult<(Route, String)> {
    let args = env::args().collect::<Vec<_>>();
    if args.len() < 3 {
        return Err(anyhow!("must provide at least 2 arguments"));
    }

    let routing_key = Route::from_str(&args[1]);
    if routing_key.is_err() {
        return Err(anyhow!("Routing key invalid"));
    };

    let routing_key = routing_key.unwrap();

    let msg = args[1..args.len()].join(" ");

    Ok((routing_key, msg))
}


#[async_std::main]
async fn main() -> Result<()> {
    setup();

    let args = parse_args();
    if args.is_err() {
        error!("{:#?}", args);
        std::process::exit(1);
    }
    let (routing_key, msg) = args.unwrap();

    let addr = std::env::var("AMQP_ADDR").unwrap_or_else(|_| LOCALHOST.into());
  
    let conn = Connection::connect(
        &addr,
        ConnectionProperties::default(),
    )
    .await?;
    info!("established connection to Rabbit server via {}", &addr);

    let channel_a = conn.create_channel().await?;
    info!("created channel");

    let exchange = channel_a.exchange_declare(
        EXCHANGE,
        EXCHANGE_TYPE,
        ExchangeDeclareOptions::default(),
        FieldTable::default()
    ).await?;
    info!(?exchange, "Declared exchange: {}", &EXCHANGE);
    
    let confirm = channel_a
        .basic_publish(
            // exchange
            EXCHANGE, 
            // routing key
            routing_key.as_ref(), 
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