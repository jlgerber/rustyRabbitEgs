use anyhow::Error as AnyhowError;
use anyhow::anyhow;
use async_std::sync::Arc;
use async_std::sync::Mutex;
use async_std::task;
use lapin::{
    options::*, publisher_confirm::Confirmation, types::FieldTable, 
    BasicProperties, Connection,Channel,
    ConnectionProperties, Result as AsyncResult
};
use lapin::types as ampt;
use std::env;
use tracing::{info,error};
use uuid::Uuid;
use rpc::{LOCALHOST, QUEUE};
use structopt::StructOpt;
use std::iter::Iterator;

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

struct Client {
    pub conn: Connection,
    pub chan: Channel,
}

impl Client {
    pub fn new() -> AsyncResult<Self> {

        task::block_on(async {
            let addr = std::env::var("AMQP_ADDR").unwrap_or_else(|_| LOCALHOST.into());
            
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

    pub fn fib(&self, input: u32) -> AsyncResult<usize> {
        task::block_on(async {
            let input = input.to_string();
            let queue = self.chan
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
            let  consumer = self.chan.basic_consume(
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
        let v: Option<usize> = None;
        let fibval = Arc::new(Mutex::new(v));

        let correlation_id = Uuid::new_v4().to_hyphenated().to_string();
        let input_c = input.clone();
        let cid = correlation_id.clone();
        let fibval_c = fibval.clone();
        let handle = task::spawn(async move {
            let mut consumer_iter = consumer.into_iter();
            while let Some(delivery_result) = consumer_iter.next() {
                if let Ok((_channel, delivery)) = delivery_result {
                   
                    if let Some(cor_id) = delivery.properties.correlation_id() {
                        if cor_id.as_str() == &cid {
                            let val = std::str::from_utf8(&delivery.data);
                            if let Ok(value) = val {
                                info!("[x] calculating fib({})", input_c);
                                let value = value.parse::<usize>().unwrap();
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

        let confirm = self.chan
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
                input.as_bytes().to_vec(),
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
            let rv = *fibval.lock().await;
            // really should create a custom error type for this
            let rv = rv.unwrap();
            Ok(rv)
        })
    }
}
 fn main() -> Result<(),AnyhowError> {
    let opts = setup();
    let client = Client::new().map_err(|e| anyhow!("{}", e))?;
    let result = client.fib(opts.guess).map_err(|e| anyhow!("{}", e))?;
    println!("fib({}) = {}",opts.guess, result);
    Ok(())
}