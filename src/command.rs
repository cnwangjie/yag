use crate::logger::Logger;
use crate::repository::get_repo;
use crate::repository::Repository;
use crate::utils::get_current_branch;
use anyhow::Result;
use clap::*;
use log::debug;

fn get_app<'a, 'b>() -> App<'a, 'b> {
    App::new("yag")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(Arg::with_name("v").short("v").help("verbose mode"))
        .subcommand(
            SubCommand::with_name("pr")
                .about("Manage requests")
                .alias("mr")
                .subcommand(SubCommand::with_name("list")
                    .about("List pull requests"))
                .subcommand(
                    SubCommand::with_name("create")
                        .alias("new")
                        .arg(Arg::with_name("title").required(true).index(1))
                        .arg(Arg::with_name("base").alias("target").long("base"))
                        .arg(
                            Arg::with_name("head")
                                .alias("source")
                                .long("head")
                                .required(true),
                        ),
                ),
        )
}

pub async fn run() -> Result<()> {
    let mut app = get_app();
    let matches = app.clone().get_matches();

    Logger::init(matches.is_present("v"))?;

    debug!("verbose mode enabled");

    if let Some(matches) = matches.subcommand_matches("pr") {
        if let Some(subcommand) = matches.subcommand_name() {
            let repo: Box<dyn Repository> = get_repo().await?;
            match subcommand {
                "list" => {
                    repo.list_pull_requests().await?;
                },
                "create" => {
                    let source_branch = match matches.value_of("head") {
                        Some(head) => head.to_string(),
                        _ => get_current_branch()?,
                    };
                    repo.create_pull_request(
                        &source_branch,
                        matches.value_of("base").unwrap_or("master"),
                        matches.value_of("title").unwrap(),
                    )
                    .await?;
                },
                _ => {

                }
            }
        } else {
            println!("{}", matches.usage());
        }
    } else {
        println!("{}", matches.usage());
    }

    Ok(())
}
