mod pr;
mod profile;

use clap::{crate_version, crate_authors, App, Arg};
use crate::logger::Logger;
use anyhow::{Result, anyhow};
use log::debug;


pub fn get_app<'a, 'b>() -> App<'a, 'b> {
  App::new("yag")
      .version(crate_version!())
      .author(crate_authors!())
      .arg(
          Arg::with_name("v")
              .short("v")
              .long("verbose")
              .help("verbose mode")
              .global(true),
      )
      .subcommand(pr::sub_command())
      .subcommand(profile::sub_command())
}


pub async fn run() -> Result<()> {
    let mut app = get_app();
    let matches = app.clone().get_matches();

    Logger::init(matches.is_present("v"))?;
    debug!("verbose mode enabled");

    let (command, arg_matches) = matches.subcommand();
    debug!("command: {}", command);
    let arg_matches = arg_matches.ok_or(anyhow!("arg matches is none"))?;
    debug!("arg matches: {:#?}", arg_matches);
    match command {
        "pr" => pr::Command::new(&arg_matches).run().await,
        "profile" => profile::Command::new(&arg_matches).run().await,
        _ => Ok(app.write_long_help(&mut std::io::stdout())?),
    }
}
