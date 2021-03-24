use crate::profile::{load_profile, prompt_add_profile, write_profile};
use anyhow::Result;
use clap::{App, ArgMatches, SubCommand};
use log::debug;

#[inline]
pub fn sub_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("profile")
        .about("Manage profiles")
        .subcommand(SubCommand::with_name("add").about("Add profile config interactively"))
}
pub struct Command<'a> {
    command: &'a str,
    matches: &'a ArgMatches<'a>,
}

impl<'a> Command<'a> {
    pub fn new(matches: &'a ArgMatches<'a>) -> Option<Self> {
        match matches.subcommand() {
            (command, Some(arg_matches)) => Some(Command {
                command: command,
                matches: arg_matches,
            }),
            _ => None,
        }
    }

    pub async fn run(&self) -> Result<()> {
        match self.command {
            "add" => self.add().await,
            _ => Ok(println!("{}", self.matches.usage())),
        }
    }

    async fn add(&self) -> Result<()> {
        let mut profile = load_profile().await?;
        debug!("profile loaded: {:#?}", profile);
        prompt_add_profile(&mut profile).await?;
        debug!("profile added: {:#?}", profile);
        write_profile(&profile).await?;
        Ok(())
    }
}
