use anyhow::{bail, Error, Result};
use clap::{App, Arg, ArgMatches, SubCommand};
use colored::Colorize;
use utils::user_input;

use crate::repository::{get_repo, ListPullRequestOpt};
use crate::utils;

#[inline]
pub fn sub_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("pr")
        .about("Manage pull requests (aka. merge request for GitLab)")
        .alias("mr")
        .subcommand(
            SubCommand::with_name("get")
                .about("Get detail of single pull request")
                .arg(Arg::with_name("id").required(true).takes_value(true)),
        )
        .subcommand(
            SubCommand::with_name("open")
                .about("Open pull request in browser")
                .arg(Arg::with_name("id").takes_value(true)),
        )
        .subcommand(
            SubCommand::with_name("close")
                .about("Close pull request")
                .arg(Arg::with_name("id").required(true).takes_value(true)),
        )
        .subcommand(
            SubCommand::with_name("list")
                .about("List pull requests")
                .about("List pull requests of current repository")
                .arg(Arg::with_name("author").long("author").takes_value(true))
                .arg(Arg::with_name("me").long("me"))
                .arg(Arg::with_name("status").long("status").takes_value(true))
                .arg(Arg::with_name("page").long("page").takes_value(true)),
        )
        .subcommand(
            SubCommand::with_name("create")
                .alias("new")
                .about("Create a new pull request")
                .arg(Arg::with_name("title").takes_value(true))
                .arg(
                    Arg::with_name("base")
                        .alias("target")
                        .long("base")
                        .short("b")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("head")
                        .alias("source")
                        .long("head")
                        .short("h")
                        .takes_value(true),
                ),
        )
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
            _ => {
                println!("{}", matches.usage());
                None
            }
        }
    }

    pub async fn run(&self) -> Result<()> {
        match self.command {
            "get" => self.get().await,
            "open" => self.open().await,
            "list" => self.list().await,
            "create" => self.create().await,
            "close" => self.close().await,
            _ => Ok(println!("{}", self.matches.usage())),
        }
    }

    async fn get(&self) -> Result<()> {
        let id = self
            .matches
            .value_of("id")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap();

        let pr = get_repo().await?.get_pull_request(id).await?;

        println!("{:#}", pr);

        Ok(())
    }

    async fn open(&self) -> Result<()> {
        let id = self
            .matches
            .value_of("id")
            .and_then(|s| s.parse::<usize>().ok());

        let repo = get_repo().await?;

        let pr = match id {
            Some(id) => repo.get_pull_request(id).await?,
            None => {
                let current_branch = utils::get_current_branch()?;
                let mut opt = ListPullRequestOpt::default();
                opt.head = Some(current_branch.to_owned());
                let result = repo.list_pull_requests(opt).await?;
                let pr = result.result.first().ok_or(anyhow::anyhow!(
                    "no pull request for current branch: {}",
                    &current_branch
                ))?;
                pr.to_owned()
            }
        };

        log::info!("opening {}", pr.url);

        open::that(pr.url)?;

        Ok(())
    }

    async fn list(&self) -> Result<()> {
        let opt = ListPullRequestOpt::from(self.matches.clone());
        let result = get_repo().await?.list_pull_requests(opt).await?;

        print!("{:#}", result);
        Ok(())
    }

    async fn create(&self) -> Result<()> {
        let source_branch = match self.matches.value_of("head") {
            Some(head) => head.to_string(),
            _ => utils::get_current_branch()?,
        };

        let target_branch = self
            .matches
            .value_of("base")
            .map(|base| base.to_string())
            .or_else(|| utils::get_git_config("yag.pr.target").ok())
            .unwrap_or("master".to_string());

        if source_branch == target_branch {
            bail!("head branch and base branch are same: {}", source_branch)
        }

        if utils::get_rev(&source_branch)? == utils::get_rev(&target_branch)? {
            let ok = user_input(&format!(
                "{} head is same as base. still create pr? (Y/n) ",
                "warning".yellow().bold()
            ))?;
            if ok == "n" {
                return Ok(());
            }
        }

        let title = self
            .matches
            .value_of("title")
            .map(|title| title.to_string())
            .or_else(|| utils::get_latest_commit_message().ok())
            .ok_or(Error::msg(
                "Cannot get latest commit message. Please specify title manually.",
            ))?;

        let pr = get_repo()
            .await?
            .create_pull_request(&source_branch, &target_branch, &title)
            .await?;
        println!("{:#}", pr);
        Ok(())
    }

    async fn close(&self) -> Result<()> {
        let id = self
            .matches
            .value_of("id")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap();
        let pr = get_repo().await?.close_pull_request(id).await?;

        println!("{:#}", pr);
        Ok(())
    }
}
