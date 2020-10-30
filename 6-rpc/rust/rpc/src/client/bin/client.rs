use anyhow::Error as AnyhowError;
use anyhow::anyhow;
use std::env;
use rpc::{  Client};
use structopt::StructOpt;

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

 fn main() -> Result<(),AnyhowError> {
    let opts = setup();
    let client = Client::new().map_err(|e| anyhow!("{}", e))?;
    let result = client.fib(opts.guess).map_err(|e| anyhow!("{}", e))?;
    println!("fib({}) = {}",opts.guess, result);
    Ok(())
}