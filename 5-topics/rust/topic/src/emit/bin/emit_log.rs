use anyhow::anyhow;
use anyhow::Result as AnyhowResult;
use lapin::{
    options::*, publisher_confirm::Confirmation, types::FieldTable, 
    BasicProperties, Connection, 
    ConnectionProperties, Result, 
};
use std::env;
use std::str::FromStr;
use tracing::{info,error};

use topic::{LOCALHOST, EXCHANGE, EXCHANGE_TYPE, RoutingKey};

// set up default logging level and initialize tracing
fn setup() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
    
    tracing_subscriber::fmt::init();
}

fn _parse_args() -> AnyhowResult<(RoutingKey, String)> {
    let args = env::args().collect::<Vec<_>>();
    if args.len() < 3 {
        return Err(anyhow!("must provide at least 2 arguments"));
    }

    let routing_key = RoutingKey::from_str(&args[1])
                                    .map_err(|_| anyhow!("unable to convert {} to RoutingKey",&args[1]))?;
    
    // if !routing_key.is_specific() {
    //     error!("invaild routing key: {:?}", routing_key);
    //     std::process::exit(1);
    // }

    let msg = args[1..args.len()].join(" ");

    Ok((routing_key, msg))
}

fn parse_args() -> (String, String) {
    match _parse_args() {
        Ok((key, msg)) => (key.to_string(), msg),
        Err(err) => {error!("Unable to parse arguments: {}", err);std::process::exit(1) ;},
    }
}

#[async_std::main]
async fn main() -> Result<()> {
    setup();

    let (routing_key,msg) = parse_args();
    

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
    info!("emitting '{}' exchange: {} routing_key: {}", &msg, &EXCHANGE, &routing_key);
    let confirm = channel_a
        .basic_publish(
            // exchange
            EXCHANGE, 
            // routing key
            &routing_key, 
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