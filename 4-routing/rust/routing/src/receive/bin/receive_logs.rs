use lapin::{
    options::*,  types::FieldTable,  Connection,
    ConnectionProperties, Result, message::DeliveryResult
};

use structopt::StructOpt;

use async_std;
use tracing::info;
use std::time;
use std::convert::AsRef;
use routing::{LOCALHOST, QUEUE, EXCHANGE, EXCHANGE_TYPE, Route};
use routing::quit_service;


#[derive(Debug, StructOpt)]
#[structopt(name="recieve-logs", about="processes messages from rabbit log queue")]
struct Opt {
    /// Optionally bound the queue size. A value of 1 ensures that the
    /// worker may only work on 1 message at a time. This will also force the 
    /// worker to work synchronously. Otherwise, all messages up to the limit
    /// specified by this flag will be processed asynchronously
    #[structopt(short="n", long="num-msgs")]
    num_msgs: Option<u16>,
    /// Add supported routing keys
    #[structopt(short="k", long="key")]
    keys: Vec<Route>
}


fn initialize() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    
    tracing_subscriber::fmt::init();
}
#[async_std::main]
async fn main() -> Result<()> {
    initialize();
    // process args
    let opt = Opt::from_args();

    let addr = std::env::var("AMQP_ADDR").unwrap_or_else(|_| LOCALHOST.into());
    // Establish connection
    let conn = Connection::connect(
        &addr,
        ConnectionProperties::default(),
    )
    .await?;

    info!("Connection to RabbitMq Server Established");

    // Create the channel
    let channel_b = conn.create_channel().await?;
    info!("Channel created");
    
    // Create the exchange from the channel
    let exchange = channel_b.exchange_declare(
        EXCHANGE,
        EXCHANGE_TYPE,
        ExchangeDeclareOptions::default(),
        FieldTable::default()
    ).await?;
    info!(?exchange, "Declared exchange: {}", &EXCHANGE);

    // declare the queue
    let mut queue_opts = QueueDeclareOptions::default();
    queue_opts.exclusive = true;
    let queue = channel_b
        .queue_declare(
            QUEUE,
            queue_opts,
            FieldTable::default(),
        )
        .await?;

    // bind the queue to the exchange
    for route in opt.keys {
        channel_b.queue_bind(
            // queue name
            &queue.name().as_str(),
            EXCHANGE,
            &route.as_ref(),
            QueueBindOptions::default(),
            FieldTable::default()
        ).await?;
        info!("Queue '{}' bound to '{}'", &queue.name().as_str(), &route.as_ref());
    }
    // Update the quality of service options to limit the consumer to
    // a specific number of messages if requested
    let qos_options = BasicQosOptions::default();
    if let Some(msgcnt) = opt.num_msgs {
        channel_b.basic_qos(msgcnt, qos_options).await?;
    }
    info!("QOS OPTIONS: {:#?}", qos_options);

    // create a consumer
    let  consumer = channel_b
        .basic_consume(
            &queue.name().as_str(),
            "my_consumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;
    info!("Channel Consumer created");
    
    // register the delegate (callback) with the consumer
    consumer.set_delegate(move |delivery: DeliveryResult| async move {
        let delivery = delivery.expect("error caught in in consumer");
        if let Some((channel, delivery)) = delivery {
            
            let val = std::str::from_utf8(&delivery.data);
            let pieces = val.split(" ");
            let level = Route::from_str(pieces[0]);
            let msg = &pieces[1..].join(" ");
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

