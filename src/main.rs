use crate::repository::get_remote_url;
use anyhow::Result;
use git_url_parse::GitUrl;
use serde_json::json;
use std::env;

mod structs;
mod utils;
mod profile;
mod gitlab;
mod repository;

use structs::*;
use utils::*;
use profile::*;
use gitlab::*;
use repository::*;


fn client() -> reqwest::Client {
    reqwest::Client::new()
}

async fn get_project_id() -> Result<u64> {
    let remote_url = get_remote_url()?;
    let remote_host = remote_url.host.expect("cannot resolve host of remote url");
    let url = format!(
        "https://{}/api/v4/projects/{}",
        remote_host,
        url_encode(remote_url.fullname)
    );
    let token = load_profile()
        .await
        ?.get_token_by_host(&remote_host)
        .expect(format!("cannot find token for {}", remote_host).as_ref());

    let res = client()
        .get(&url)
        .header("Private-Token", token)
        .send()
        .await?;

    let data = res.json::<Project>().await?;

    Ok(data.id)
}

async fn create_merge_request() -> Result<()> {
    let remote_url = get_remote_url()?;
    let remote_host = remote_url.host.expect("cannot resolve host of remote url");
    let url = format!(
        "https://{}/api/v4/projects/{}/merge_requests",
        remote_host,
        get_project_id().await?
    );

    let token = load_profile()
        .await
        ?.get_token_by_host(&remote_host)
        .expect(format!("cannot find token for {}", remote_host).as_ref());

    let res = client()
        .post(&url)
        .header("Private-Token", token)
        .header("Content-Type", "application/json")
        .body(
            json!({
                "source_branch": "dev",
                "target_branch": "master",
                "title": "ttt",
            })
            .to_string(),
        )
        .send()
        .await?;

    let data = res.text().await?;
    println!("{:#?}", data);
    Ok(())
}

async fn list_merge_requests() -> Result<()> {
    let remote_url = get_remote_url()?;
    let remote_host = remote_url.host.expect("cannot resolve host of remote url");
    let url = format!(
        "https://{}/api/v4/projects/{}/merge_requests?state=opened",
        remote_host,
        get_project_id().await?
    );

    let token = load_profile()
        .await
        ?.get_token_by_host(&remote_host)
        .expect(format!("cannot find token for {}", remote_host).as_ref());

    let res = client()
        .get(&url)
        .header("Private-Token", token)
        .send()
        .await?;

    let data = res.json::<Vec<MergeRequest>>().await?;
    println!("{:#?}", data);

    Ok(())
}

async fn mr() {
    create_merge_request().await.expect("failed to create mr");
}

async fn usage() {
    println!("Usage: yag COMMAND [args]");
}

async fn run() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    let repo: Box<dyn Repository> = get_repo().await?;

    match args.get(1).unwrap_or(&"".to_string()).as_ref() {
        "mrlist" => repo.list_pull_requests()?,
        "mr" => repo.create_pull_request()?,
        "pr" => repo.create_pull_request()?,
        _ => usage().await,
    };

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    run().await?;
    Ok(())
}
