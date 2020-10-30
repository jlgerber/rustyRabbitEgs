use async_std;
use async_std::task;
use lapin::{
    options::*,  types::FieldTable,  Connection,BasicProperties,
    ConnectionProperties, Result as AsyncResult, message::DeliveryResult, Queue, Channel
};
use std::env;
use structopt::StructOpt;
use tracing::info;

use rpc::{LOCALHOST, QUEUE, ffib, SimpleClient};
use rpc::quit_service;
use anyhow::anyhow;
use anyhow::Error as AnyhowError;


#[derive(Debug, StructOpt)]
#[structopt(name="server", about="processes messages from rabbit work queue")]
struct Opt {
    /// Optionally bound the queue size. A value of 1 ensures that the
    /// worker may only work on 1 message at a time. This will also force the 
    /// worker to work synchronously. Otherwise, all messages up to the limit
    /// specified by this flag will be processed asynchronously
    #[structopt(short="n", long="num-msgs")]
    num_msgs: Option<u16>
}

// Perform basic setup, including parsing arguments
fn setup() -> Opt {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
    
    tracing_subscriber::fmt::init();
    Opt::from_args()
}

pub struct Server {
    pub queue_name: String,
    pub msgcnt: Option<u16>,
    pub inner: SimpleClient,
    pub queue_declare_opts: QueueDeclareOptions,
    pub qos_opts: BasicQosOptions,
    pub consume_opts: BasicConsumeOptions
}

impl Server {
    pub fn new(
        queue_name: impl Into<String>,
        msgcnt: Option<u16>,
        queue_declare_opts: QueueDeclareOptions,
        qos_opts: BasicQosOptions,
        consume_opts: BasicConsumeOptions
    ) -> AsyncResult<Self> {
        let client = SimpleClient::new()?;
        let name = queue_name.into();
        Ok(Server {
            queue_name: name,
            msgcnt,
            inner: client,
            queue_declare_opts,
            qos_opts,
            consume_opts
        })
    }

    /// Create  a Server instance with defualt values for Optionals. This
    /// cannot be implemented via Default trait, as it is fallible.
    pub fn with_defaults(queue_name: impl Into<String>) -> AsyncResult<Self> {
        Self::new(
            queue_name,
            None,
            QueueDeclareOptions{
                durable: false,
                exclusive:false,
                auto_delete: false, 
                nowait: false,
                ..Default::default()
            },
            BasicQosOptions{global: false, ..Default::default()},
            BasicConsumeOptions{
                no_local: false,
                no_ack: false,
                exclusive: false,
                nowait: false
            },
        )
    }

    pub fn queue_name(&mut self, name: impl Into<String>) {
        self.queue_name = name.into();
    }
    pub fn message_count(&mut self, cnt: Option<u16>) {
        self.msgcnt = cnt
    }
    /// Set queue declare options
    pub fn queue_declare_opts(&mut self, options: QueueDeclareOptions) {
        self.queue_declare_opts = options;
    }
    /// Set qos options
    pub fn qos_options(&mut self, options: BasicQosOptions) {
        self.qos_opts = options;
    }
    /// Set consume options
    pub fn consume_opts(&mut self, options: BasicConsumeOptions) {
        self.consume_opts = options;
    }
    /// Initiate the async service.
    pub fn serve(&self) -> AsyncResult<()> {
        task::block_on(async {

            let queue = self.inner.chan
                    .queue_declare(
                        self.queue_name.as_str(),
                        self.queue_declare_opts.clone(),
                        FieldTable::default(),
                    )
                    .await?;

            info!(?queue, "Declared queue '{}'", &queue.name().as_str());

            // Set the number of messages that the channel's consumer can process
            // at one time

            //let qos_options = BasicQosOptions{global: false, ..Default::default()};
            let qos_options = self.qos_opts.clone();
            if let Some(msgcnt) = self.msgcnt {
                self.inner.chan.basic_qos(msgcnt, qos_options).await?;
            } else {
                self.inner.chan.basic_qos(1, qos_options).await?;
            }
            info!("QOS OPTIONS: {:#?}", qos_options);

            // Create the consumer for the incoming. named queue
            let  consumer = self.inner.chan
                .basic_consume(
                    self.queue_name.as_str(),
                    "",
                    self.consume_opts.clone(),
                    FieldTable::default(),
                )
                .await?;

            info!("Channel Consumer created");

            let handle = task::spawn(async move {
                //let delivery = delivery.expect("error caught in in consumer");
                let mut consumer_iter = consumer.into_iter();
                while let Some(delivery_result) = consumer_iter.next() {
                    if let Ok((channel, delivery)) = delivery_result {
                        // perhaps we can use bytevec crate to 
                        // skip the string
                        let val = std::str::from_utf8(&delivery.data);
                        
                        if let Ok(value) = val {
                            if let Ok(intvalue) = value.parse::<u32>() {
                                println!("[.] Calculating fib({})", intvalue);
                                let result = ffib(intvalue as usize);
                                println!("[X] fib({}) = {}",intvalue, result);
                                let cid = delivery.properties.correlation_id().clone().unwrap();
                                channel.basic_publish(
                                    "", //exchange
                                    delivery.properties.reply_to().as_ref().unwrap().as_str(),
                                    BasicPublishOptions::default(),
                                    result.to_string().as_bytes().to_vec(),
                                    BasicProperties::default().with_correlation_id(cid)
                                ).await.unwrap();
                            } else {
                                println!("ERROR: Unable to convert {} to an int", &value);
                            }
                            
                        } else {
                            println!("unable to convert raw data from delivery to string");
                        }
                        channel
                            .basic_ack(delivery.delivery_tag, BasicAckOptions::default())
                            .await
                            .expect("failed to ack");
                    }
                } 

            });
            handle.await;

            Ok(())
        })
    }
}

#[async_std::main]
async fn main() -> AsyncResult<()> {

    // process args
    let opt = setup();
    let mut server = Server::with_defaults(QUEUE)?; 
    server.message_count(opt.num_msgs);
    server.serve()?;
    Ok(())
}

