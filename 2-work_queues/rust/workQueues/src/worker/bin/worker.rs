use lapin::{
    options::*,  types::FieldTable,  Connection,
    ConnectionProperties, Result, message::DeliveryResult
};

use structopt::StructOpt;

use async_std;
use tracing::info;
use std::time;

use work_queues::{LOCALHOST, QUEUE};
use work_queues::quit_service;


#[derive(Debug, StructOpt)]
#[structopt(name="worker", about="processes messages from rabbit work queue")]
struct Opt {
    /// Optionally bound the queue size. A value of 1 ensures that the
    /// worker may only work on 1 message at a time. This will also force the 
    /// worker to work synchronously. Otherwise, all messages up to the limit
    /// specified by this flag will be processed asynchronously
    #[structopt(short="n", long="num-msgs")]
    num_msgs: Option<u16>
}


#[async_std::main]
async fn main() -> Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    // process args
    let opt = Opt::from_args();

    tracing_subscriber::fmt::init();

    let addr = std::env::var("AMQP_ADDR").unwrap_or_else(|_| LOCALHOST.into());

    // Establish connection
    let conn = Connection::connect(
        &addr,
        ConnectionProperties::default(),
    )
    .await?;

    info!("Connection to RabbitMq Server Established");

    let channel_b = conn.create_channel().await?;
    info!("Channel created");
    
    let _queue = channel_b
        .queue_declare(
            QUEUE,
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;
    // qos_options must use interior mutability.
    let qos_options = BasicQosOptions::default();
    if let Some(msgcnt) = opt.num_msgs {
        channel_b.basic_qos(msgcnt, qos_options).await?;
    }

    info!("QOS OPTIONS: {:#?}", qos_options);

    let  consume_options = BasicConsumeOptions::default();
    // exclusive queue 
    //consume_options.exclusive= true;
    // doesn't need to be mutable anymore
    let consume_options = consume_options;

    info!("consume options {:#?}", consume_options);
    let  consumer = channel_b
        .basic_consume(
            QUEUE,
            "my_consumer",
            consume_options,
            FieldTable::default(),
        )
        .await?;

    info!("Channel Consumer created");
    consumer.set_delegate(move |delivery: DeliveryResult| async move {
        let delivery = delivery.expect("error caught in in consumer");
        if let Some((channel, delivery)) = delivery {
            
            let val = std::str::from_utf8(&delivery.data);
            
            if let Ok(value) = val {
                println!("[x] Start:  {}",value);
                let sleep_duration = value.matches('.').count();
                if sleep_duration > 0 {
                async_std::task::sleep(time::Duration::new(sleep_duration as u64,0)).await;
            }
                println!("[x] Finish: {}",value);
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

