use async_std::task;
use lapin::{
    BasicProperties,
    options::*, 
    Result as AsyncResult,
    types::FieldTable, 
};
use std::iter::Iterator;
use tracing::{info};

use crate::{SimpleClient, fib};


/// Server which receives messages over RabbitMq which each provide an
/// index into fibonacci sequence, which the service calculates and
/// returns over a reply channel provided in the message.
pub struct FibRpcServer {
    queue_name: String,
    msgcnt: Option<u16>,
    inner: SimpleClient,
    queue_declare_opts: QueueDeclareOptions,
    qos_opts: BasicQosOptions,
    consume_opts: BasicConsumeOptions
}

impl FibRpcServer {
    /// Create a new instance of the FibRpcServer, given a queue name, optional message count,
    /// and various lapin options
    pub fn new(
        queue_name: impl Into<String>,
        msgcnt: Option<u16>,
        queue_declare_opts: QueueDeclareOptions,
        qos_opts: BasicQosOptions,
        consume_opts: BasicConsumeOptions
    ) -> AsyncResult<Self> {
        let client = SimpleClient::new()?;
        let name = queue_name.into();
        Ok(FibRpcServer {
            queue_name: name,
            msgcnt,
            inner: client,
            queue_declare_opts,
            qos_opts,
            consume_opts
        })
    }

    /// Create  a FibRpcServer instance with default values for Optionals. 
    ///
    /// This cannot be implemented via Default trait, as it is fallible.
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

    /// retrieve a reference to the inner SimpleClient instance, which houses
    /// the connection and channel
    pub fn client(&self) -> &SimpleClient {
        &self.inner
    }
    /// Set the queue name after  the fact.
    pub fn set_queue_name(&mut self, name: impl Into<String>) {
        self.queue_name = name.into();
    }
    /// Retrieve the queue name as a &str
    pub fn queue_name(&self) -> &str {
        &self.queue_name
    }
    /// Set the total number of messages which may be processed
    /// at once. If None is supplied, there are no limits
    pub fn set_message_count(&mut self, cnt: Option<u16>) {
        self.msgcnt = cnt
    }
    /// Retrieve the message count
    pub fn message_count(&self) -> Option<u16> {
        self.msgcnt
    }
    /// Set queue declare options
    pub fn set_queue_declare_opts(&mut self, options: QueueDeclareOptions) {
        self.queue_declare_opts = options;
    }
    /// Retrieve a reference to the QueueDeclareOptions struct
    pub fn queue_declare_opts(&self) -> &QueueDeclareOptions {
        &self.queue_declare_opts
    }
    /// Set qos options
    pub fn set_qos_options(&mut self, options: BasicQosOptions) {
        self.qos_opts = options;
    }
    /// Retrieve a reference to the BasicQosOptions 
    pub fn qos_options(&self)-> &BasicQosOptions {
        &self.qos_opts
    }
    /// Set consume options
    pub fn set_consume_opts(&mut self, options: BasicConsumeOptions) {
        self.consume_opts = options;
    }
    /// Retrieve a reference to the BasicConsumeOptions
    pub fn consume_opts(&self) -> &BasicConsumeOptions {
        &self.consume_opts
    }
    /// Start the service up. This method will block until done
    pub fn serve(&self) -> AsyncResult<()> {
        task::block_on(async {

            let queue = self.client().chan
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
                self.client().chan.basic_qos(msgcnt, qos_options).await?;
            } else {
                self.client().chan.basic_qos(1, qos_options).await?;
            }
            info!("QOS OPTIONS: {:#?}", qos_options);

            // Create the consumer for the incoming. named queue
            let  consumer = self.client().chan
                .basic_consume(
                    self.queue_name.as_str(),
                    "",
                    self.consume_opts.clone(),
                    FieldTable::default(),
                )
                .await?;

            info!("Channel Consumer created");
            // this will automagically move long running jobs onto a separate
            // thread if they take too long. thanks async_std.
            let handle = task::spawn(async move {
            
                let mut consumer_iter = consumer.into_iter();
                while let Some(delivery_result) = consumer_iter.next() {
                    if let Ok((channel, delivery)) = delivery_result {
                        // perhaps we can use bytevec crate to 
                        // skip the string
                        let val = std::str::from_utf8(&delivery.data);
                        
                        if let Ok(value) = val {
                            if let Ok(intvalue) = value.parse::<u32>() {
                                println!("[.] Calculating fib({})", intvalue);
                                let result = fib(intvalue as usize);
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