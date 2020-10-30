use async_std;
use lapin::{
    options::*,  types::FieldTable,  Connection,BasicProperties,
    ConnectionProperties, Result, message::DeliveryResult
};
use std::env;
use structopt::StructOpt;
use tracing::info;

use rpc::{LOCALHOST, QUEUE};
use rpc::quit_service;



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

// // very inefficent way of calculating fibonacci series
// fn fib(n: usize) -> usize {
//     if n == 0 {
//         0
//     } else if n == 1{
//         1
//     } else {
//         fib(n-1) + fib(n-2)
//     }
// }

fn _ffib(n: usize) -> (usize, usize) {
    if n == 0 {
        (0,1)
    } else {
        let (a,b) = _ffib(n / 2);
        let c = a * (b * 2 - a);
        let d = a * a + b * b;
        if n % 2 == 0{
            (c,d)
        } else {
            (d, c + d)
        }
    }
}

fn ffib(n:usize) -> usize {
    if n == 0 {
        0 
    } else {
        let (_,b) = _ffib(n-1);
        b
    }
}

// Perform basic setup, including parsing arguments
fn setup() -> Opt {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
    
    tracing_subscriber::fmt::init();
    Opt::from_args()
}


#[async_std::main]
async fn main() -> Result<()> {

    // process args
    let opt = setup();
    let addr = std::env::var("AMQP_ADDR").unwrap_or_else(|_| LOCALHOST.into());

    // Establish connection
    let conn = Connection::connect(
        &addr,
        ConnectionProperties::default(),
    )
    .await?;

    info!("Connection to RabbitMq Server Established");

    let channel = conn.create_channel().await?;
    info!("Channel created");
    
    // generate a queue from the channel
    let queue = channel
                .queue_declare(
                    QUEUE,
                    // this is just to illustrate the settings
                    // we could have simply called QueueDeclareOptions::default()
                    QueueDeclareOptions{
                        durable: false,
                        exclusive:false,
                        auto_delete: false, 
                        nowait: false,
                        ..Default::default()
                    },
                    FieldTable::default(),
                )
                .await?;

info!(?queue, "Declared queue '{}'", &queue.name().as_str());

    // Set the number of messages that the channel's consumer can process
    // at one time
    let qos_options = BasicQosOptions{global: false, ..Default::default()};
    if let Some(msgcnt) = opt.num_msgs {
        channel.basic_qos(msgcnt, qos_options).await?;
    } else {
        channel.basic_qos(1, qos_options).await?;
    }
    info!("QOS OPTIONS: {:#?}", qos_options);

    // Create the consumer for the incoming. named queue
    let  consumer = channel
        .basic_consume(
            QUEUE,
            "",
            BasicConsumeOptions{
                no_local: false,
                no_ack: false,
                exclusive: false,
                nowait: false
            },
            FieldTable::default(),
        )
        .await?;

    info!("Channel Consumer created");

    consumer.set_delegate(move |delivery: DeliveryResult| async move {
        let delivery = delivery.expect("error caught in in consumer");
        if let Some((channel, delivery)) = delivery {
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

    })?;
    
    Ok(quit_service::prompt())
   
}

