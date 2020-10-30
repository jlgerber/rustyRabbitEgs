use async_std::sync::Arc;
use async_std::sync::Mutex;
use async_std::task;
use lapin::{types as ampt,
    options::*, publisher_confirm::Confirmation, types::FieldTable, 
    BasicProperties,
     Connection,Channel,
    ConnectionProperties, Result as AsyncResult
};
use std::env;
use std::iter::Iterator;
use tracing::{info, error};
use uuid::Uuid;

pub mod fib;
pub use fib::ffib;

pub const LOCALHOST: &'static str = "amqp://127.0.0.1:5672/%2f";
pub const QUEUE: &'static str = "rpc_queue_rust";

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


pub struct Client {
    inner: SimpleClient
}

impl Client {
    pub fn new() -> AsyncResult<Self> {
        let inner = SimpleClient::new()?;
        Ok(Self{inner})
    }

    pub fn fib(&self, input: u32) -> AsyncResult<usize> {
        task::block_on(async {
            let input = input.to_string();
            let queue = self.inner.chan
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
            let  consumer = self.inner.chan.basic_consume(
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
                                    // this is where the break should be
                                    break;
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
                    //break;
                }

            });

            let confirm = self.inner.chan
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