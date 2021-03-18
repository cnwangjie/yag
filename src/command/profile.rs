use clap::{App, ArgMatches, SubCommand};
use anyhow::Result;
use crate::profile::{load_profile, prompt_add_profile};

#[inline]
pub fn sub_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("profile")
        .about("Manage profiles")
        .subcommand(
            SubCommand::with_name("add")
                .about("Add profile config interactively"),
        )
}
pub struct Command<'a> {
    matches: &'a ArgMatches<'a>,
}

impl<'a> Command<'a> {
    pub fn new(matches: &'a ArgMatches<'a>) -> Self {
        Command {
            matches: matches,
        }
    }

    pub async fn run(&self) -> Result<()> {
        let (command, _) = self.matches.subcommand();
        match command {
            "add" => self.add().await,
            _ => Ok(println!("{}", self.matches.usage())),
        }
    }

    async fn add(&self) -> Result<()> {
        let mut profile = load_profile().await?;
        prompt_add_profile(&mut profile).await?;
        Ok(())
    }
}
