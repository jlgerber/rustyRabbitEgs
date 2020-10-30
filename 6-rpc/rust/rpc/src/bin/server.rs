use async_std;
use lapin::Result as AsyncResult;
use std::env;
use structopt::StructOpt;

use rpc::{ QUEUE, FibRpcServer};


#[derive(Debug, StructOpt)]
#[structopt(name="server", about="processes messages from rabbit work queue")]
struct Opt {
    /// Optionally bind the queue size. A value of 1 ensures that the
    /// server may only work on 1 message at a time. This will also force the 
    /// server to work synchronously. Otherwise, all messages up to the limit
    /// specified by this flag will be processed asynchronously
    #[structopt(short="n", long="num-msgs")]
    num_msgs: Option<u16>,
    /// set the log level
    #[structopt(short="l", long="log-level")]
    loglevel: Option<String>
}

// Perform basic setup, including parsing arguments
fn setup() -> Opt {

    let mut args = Opt::from_args();
    if args.loglevel.is_some() {
        let level = args.loglevel.take().unwrap();
        env::set_var("RUST_LOG", &level);
    } else if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "warn");
    }
    tracing_subscriber::fmt::init();
    
    args
}


#[async_std::main]
async fn main() -> AsyncResult<()> {
    // process args
    let opt = setup();
    let mut server = FibRpcServer::with_defaults(QUEUE)?; 
    server.set_message_count(opt.num_msgs);
    server.serve()?;
    Ok(())
}

