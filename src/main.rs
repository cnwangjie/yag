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
    run().await?;
    Ok(())
}
