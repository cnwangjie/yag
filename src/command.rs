use crate::logger::Logger;
use crate::repository::get_repo;
use crate::repository::ListPullRequestOpt;
use crate::profile::{prompt_add_profile, load_profile};
use crate::utils;
use anyhow::{Error, Result};
use clap::*;
use log::debug;

fn get_app<'a, 'b>() -> App<'a, 'b> {
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
        .subcommand(
            SubCommand::with_name("pr")
                .about("Manage pull requests (aka. merge request for GitLab)")
                .alias("mr")
                .subcommand(
                    SubCommand::with_name("get")
                        .about("Get detail of single pull request")
                        .arg(Arg::with_name("id").required(true).takes_value(true)),
                )
                .subcommand(
                    SubCommand::with_name("close")
                        .about("Close pull request")
                        .arg(Arg::with_name("id").required(true).takes_value(true)),
                )
                .subcommand(
                    SubCommand::with_name("list").about("List pull requests")
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
                ),
        )
        .subcommand(
            SubCommand::with_name("profile")
                .about("Manage profiles")
                .subcommand(
                    SubCommand::with_name("add")
                        .about("Add profile config interactively"),
                )
        )
}

pub async fn run() -> Result<()> {
    let app = get_app();
    let matches = app.clone().get_matches();

    Logger::init(matches.is_present("v"))?;
    debug!("verbose mode enabled");

    if let Some(matches) = matches.subcommand_matches("pr") {
        if let (subcommand, Some(matches)) = matches.subcommand() {
            let repo = get_repo().await?;
            match subcommand {
                "get" => {
                    let id = matches
                        .value_of("id")
                        .and_then(|s| s.parse::<usize>().ok())
                        .unwrap();
                    let pr = repo.get_pull_request(id).await?;
                    println!("{:#}", pr);
                }
                "list" => {
                    let opt = ListPullRequestOpt::from(matches.clone());
                    let result = repo.list_pull_requests(opt).await?;
                    print!("{:#}", result);
                }
                "create" => {
                    let source_branch = match matches.value_of("head") {
                        Some(head) => head.to_string(),
                        _ => utils::get_current_branch()?,
                    };

                    let target_branch = matches.value_of("base")
                        .map(|base| base.to_string())
                        .or_else(|| {
                            utils::get_git_config("yag.pr.target").ok()
                        })
                        .unwrap_or("master".to_string());

                    let title = matches
                        .value_of("title")
                        .map(|title| title.to_string())
                        .or_else(|| {
                            utils::get_latest_commit_message().ok()
                        })
                        .ok_or(Error::msg("Cannot get latest commit message. Please specify title manually."))?;

                    let pr = repo.create_pull_request(
                            &source_branch,
                            &target_branch,
                            &title,
                        )
                        .await?;
                    print!("{:#}", pr);
                }
                "close" => {
                    let id = matches
                        .value_of("id")
                        .and_then(|s| s.parse::<usize>().ok())
                        .unwrap();
                    let pr = repo.close_pull_request(id).await?;
                    print!("{:#}", pr);
                }
                _ => {
                }
            }
        } else {
            println!("{}", matches.usage());
        }
    } else if let Some(matches) = matches.subcommand_matches("profile") {
        let (subcommand, _) = matches.subcommand();

        match subcommand {
            "add" => {
                let mut profile = load_profile().await?;
                prompt_add_profile(&mut profile).await?;
            }
            _ => {
                println!("{}", matches.usage());
            }
        }
    } else {
        println!("{}", matches.usage());
    }

    Ok(())
}
