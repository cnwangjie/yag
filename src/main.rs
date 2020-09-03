use std::env;
use std::process::Command;
use std::ffi::OsStr;
use serde_json::json;
use git_url_parse::GitUrl;
use anyhow::Result;

mod utils;
mod structs;

use utils::*;
use structs::*;

fn spawn(command: &str) -> Result<String> {
    let mut parts = command.split(' ');
    let program = parts.next().unwrap();
    let args: Vec<&OsStr> = parts.map(OsStr::new).collect();

    let buf = Command::new(program)
        .args(args)
        .output()
        ?.stdout;

    let result = String::from_utf8(buf)?;

    Ok(result)
}

fn get_remote_url() -> Result<GitUrl> {
    let remote_url = spawn("git remote get-url origin")?;
    Ok(GitUrl::parse(remote_url.trim())?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn() {
        assert_eq!(spawn("echo 123"), "123\n")
    }

    #[test]
    fn test_get_remote_host() {
        println!("{}", get_remote_url().unwrap());
    }
}

fn client() -> reqwest::Client {
    reqwest::Client::new()
}

static TOKEN: &str = "gizdyLGuLxWFA673sgT_";

async fn get_project_id() -> Result<u64> {
    let remote_url = get_remote_url()?;
    let remote_host = remote_url.host.expect("cannot resolve host of remote url");
    let url = format!("https://{}/api/v4/projects/{}", remote_host, url_encode(remote_url.fullname));
    let res = client().get(&url)
        .header("Private-Token", TOKEN)
        .send()
        .await?;

    let data = res.json::<Project>().await?;

    Ok(data.id)
}

async fn create_merge_request() -> Result<()> {
    let remote_url = get_remote_url()?;
    let remote_host = remote_url.host.expect("cannot resolve host of remote url");
    let url = format!("https://{}/api/v4/projects/{}/merge_requests", remote_host, get_project_id().await?);
    let res = client().post(&url)
        .header("Private-Token", TOKEN)
        .header("Content-Type", "application/json")
        .body(json!({
            "source_branch": "dev",
            "target_branch": "master",
            "title": "ttt",
        }).to_string())
        .send()
        .await?;

    let data = res.text().await?;
    println!("{:#?}", data);
    Ok(())
}

async fn list_merge_requests() -> Result<()> {
    let remote_url = get_remote_url()?;
    let remote_host = remote_url.host.expect("cannot resolve host of remote url");
    let url = format!("https://{}/api/v4/projects/{}/merge_requests?state=opened", remote_host, get_project_id().await?);
    let res = client().get(&url)
        .header("Private-Token", TOKEN)
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

async fn run() {
    let args: Vec<String> = env::args().collect();
    match args.get(1).unwrap_or(&"".to_string()).as_ref() {
        "mrlist" => list_merge_requests().await.expect("failed to list mr"),
        "mr" => mr().await,
        "pr" => mr().await,
        _ => usage().await,
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    run().await;
    Ok(())
}
