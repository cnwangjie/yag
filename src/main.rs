use anyhow::Result;

mod command;
mod gitlab;
mod logger;
mod profile;
mod repository;
mod structs;
mod utils;

use command::run;

#[tokio::main]
async fn main() -> Result<()> {
    let code = run().await.and(Ok(0)).unwrap_or_else(|err| {
        eprintln!("Error: {}", err);
        -1
    });
    std::process::exit(code);
}
