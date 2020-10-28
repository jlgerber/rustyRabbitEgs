use lapin::{
    options::*, publisher_confirm::Confirmation, types::FieldTable, 
    BasicProperties, Connection,
    ConnectionProperties, Result, message::DeliveryResult,
};
use lapin::types as ampt;
use std::env;
use tracing::info;
use uuid::Uuid;
use rpc::quit_service;
use rpc::{LOCALHOST, QUEUE};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name="quess", about="provide a fib index")]
struct Opt {
    /// Index of the value in the fibonacci sequence to 
    /// calculate
    #[structopt(name = "GUESS")]
    guess: u32
}

fn setup() -> Opt {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
    if env::args().len() < 2 {
        println!("Usage: client <message>");
        std::process::exit(1);
    }
    tracing_subscriber::fmt::init();
    Opt::from_args()
}


#[async_std::main]
async fn main() -> Result<()> {
    let opts = setup();
    let guess = opts.guess.to_string();
    
    let addr = std::env::var("AMQP_ADDR").unwrap_or_else(|_| LOCALHOST.into());

  
    let conn = Connection::connect(
        &addr,
        ConnectionProperties::default(),
    )
    .await?;
    info!("established connection to Rabbit server via {}", &addr);

    let channel = conn.create_channel().await?;
    info!("created channel");

    let queue = channel
        .queue_declare(
            "",
            QueueDeclareOptions{
                durable: false,
                exclusive:true,
                auto_delete: false, 
                nowait: false,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await?;
  
    info!(?queue, "Declared queue 'hello'");
    
    // consumer
    let consumer = channel.basic_consume(
        queue.name().as_str(), //queue
        "", // consumer tag
        BasicConsumeOptions{
            no_local: false,
            no_ack: true,
            exclusive: false,
            nowait: false,
        }, 
        FieldTable::default()
    ).await?;

 
    // generate correlation id
    
    let correlation_id = Uuid::new_v4().to_hyphenated().to_string();
    consumer.set_delegate(move |delivery: DeliveryResult| async move {
        let delivery = delivery.expect("error caught in in client");
        if let Some((_channel, delivery)) = delivery {
            
            let val = std::str::from_utf8(&delivery.data);
            if let Ok(value) = val {
                println!("[x] {}: f() = {}",delivery.routing_key, value);
               
            } else {
                println!("unable to convert raw data from delivery to string");
            }
        } 

    })?;

    let confirm = channel
        .basic_publish(
            // exchange
            "", 
            // routing key
            QUEUE, 
            // options
            BasicPublishOptions{
                mandatory: false,
                immediate: false,
            },
            // payload
            guess.as_bytes().to_vec(),
            // properties
            BasicProperties::default()
                .with_content_type(ampt::ShortString::from("text/plain"))
                .with_correlation_id(ampt::ShortString::from(correlation_id))
                .with_reply_to(queue.name().clone())
                ,
         )
        .await?
        .await?; // two awaits to get from a doubly wrapped
        // Result 
    assert_eq!(confirm, Confirmation::NotRequested);
        
    Ok(quit_service::prompt())
}