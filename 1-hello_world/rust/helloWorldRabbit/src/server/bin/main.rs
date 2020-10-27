
use async_std;
use lapin::{
    options::*,  types::FieldTable,  Connection,
    ConnectionProperties, Result, message::DeliveryResult
};
use tracing::info;

use hello_world::quit_service;
use hello_world::LOCALHOST;

#[async_std::main]
async fn main() -> Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

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
    
    let  consumer = channel_b
        .basic_consume(
            "hello",
            "my_consumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    info!("Channel Consumer created");
    consumer.set_delegate(move |delivery: DeliveryResult| async move {
        let delivery = delivery.expect("error caught in in consumer");
        if let Some((channel, delivery)) = delivery {
            channel
                .basic_ack(delivery.delivery_tag, BasicAckOptions::default())
                .await
                .expect("failed to ack");
            let val = std::str::from_utf8(&delivery.data);
            if let Ok(value) = val {

                println!("[x] Delivered: {}",value);
            } else {
                println!("unable to convert raw data from delivery to string");
            }
        } 

    })?;
    
    Ok(quit_service::prompt())
}
