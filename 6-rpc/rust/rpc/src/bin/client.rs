use anyhow::Error as AnyhowError;
use anyhow::anyhow;
use std::env;
use rpc::FibClient;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name="quess", about="provide a fib index")]
struct Opt {
    /// Index of the value in the fibonacci sequence to 
    /// calculate
    #[structopt(name = "GUESS")]
    guess: u32,
    /// set the log level
    #[structopt(short="l", long="log-level")]
    loglevel: Option<String>
}

// parse args, initialize the log level, and start
// the tracing
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

 fn main() -> Result<(),AnyhowError> {
    let opts = setup();
    let client = FibClient::new().map_err(|e| anyhow!("{}", e))?;
    let result = client.fib(opts.guess).map_err(|e| anyhow!("{}", e))?;
    println!("fib({}) = {}",opts.guess, result);
    Ok(())
}