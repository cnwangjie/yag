mod pr;
mod profile;

use crate::logger::Logger;
use anyhow::Result;
use clap::{crate_authors, crate_version, App, AppSettings, Arg};
use log::debug;

pub fn get_app<'a, 'b>() -> App<'a, 'b> {
    App::new("yag")
        .version(crate_version!())
        .author(crate_authors!())
        .global_setting(AppSettings::ColoredHelp)
        .global_setting(AppSettings::DisableHelpSubcommand)
        .global_setting(AppSettings::DontCollapseArgsInUsage)
        .global_setting(AppSettings::VersionlessSubcommands)
        .arg(
            Arg::with_name("v")
                .short("v")
                .long("verbose")
                .help("verbose mode")
                .global(true),
        )
        .subcommand(pr::sub_command().setting(AppSettings::SubcommandRequiredElseHelp))
        .subcommand(profile::sub_command().setting(AppSettings::SubcommandRequiredElseHelp))
}

pub async fn run() -> Result<()> {
    let mut app = get_app();

    let matches = app.clone().get_matches();

    Logger::init(matches.is_present("v"))?;
    debug!("verbose mode enabled");

    if let (command, Some(arg_matches)) = matches.subcommand() {
        debug!("command: {}", command);
        debug!("arg matches: {:#?}", arg_matches);
        match command {
            "pr" => pr::Command::new(&arg_matches).unwrap().run().await?,
            "profile" => profile::Command::new(&arg_matches).unwrap().run().await?,
            _ => (),
        }
    } else {
        app.print_long_help()?;
        println!();
    }

    Ok(())
}
