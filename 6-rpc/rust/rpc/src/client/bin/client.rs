use async_std::sync::Arc;
use async_std::sync::Mutex;
use lapin::{
    options::*, publisher_confirm::Confirmation, types::FieldTable, 
    BasicProperties, Connection,
    ConnectionProperties, Result
};
use lapin::types as ampt;
use std::env;
use tracing::{info,error};
use uuid::Uuid;
use rpc::{LOCALHOST, QUEUE};
use structopt::StructOpt;
use std::iter::Iterator;
use async_std::task;

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
  
    info!( "Declared queue");
    
    // consumer
    let  consumer = channel.basic_consume(
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
    let v: Option<u32> = None;
    let fibval = Arc::new(Mutex::new(v));

    let correlation_id = Uuid::new_v4().to_hyphenated().to_string();
    let guess_c = guess.clone();
    let cid = correlation_id.clone();
    let fibval_c = fibval.clone();
    let handle = task::spawn(async move {
        let mut consumer_iter = consumer.into_iter();
        while let Some(delivery_result) = consumer_iter.next() {
            if let Ok((_channel, delivery)) = delivery_result {
                // to be certain we have to check the correlation _id
                //let val = std::str::from_utf8(&delivery.data);
                // if let Ok(value) = val {
                //     println!("[x] calculating fib({})", guess_c);
                //     let value = value.parse::<u32>().unwrap();
                //     *fibval_c.lock().await = Some(value);
                // } else {
                //     println!("unable to convert raw data from delivery to string");
                // }

                // do a bit more work to get at the correlation_id 
                // (ok a lot more work)
                if let Some(cor_id) = delivery.properties.correlation_id() {
                    if cor_id.as_str() == &cid {
                        let val = std::str::from_utf8(&delivery.data);
                        if let Ok(value) = val {
                            info!("[x] calculating fib({})", guess_c);
                            let value = value.parse::<u32>().unwrap();
                            *fibval_c.lock().await = Some(value);
                        } else {
                            error!("unable to convert raw data from delivery to string");
                        }
                    } else {
                        error!("Correlation ids do not match");
                    }
                } else {
                    error!("Error unwrapping correlation id {:#?}", delivery.properties.correlation_id());
                }
               
            } else {
                error!("unable to convert raw data from delivery to string");
            }
            break;
        }

    });

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
    
    handle.await;
    match *fibval.lock().await {
        Some(val) => println!("\n\tfib() == {}", val),
        None => error!("Error trying to calculate fib")
    }
    Ok(())
}