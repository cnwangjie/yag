use crate::gitlab::GitLabRepository;
use crate::utils::spawn;
use anyhow::*;
use async_trait::async_trait;
use git_url_parse::GitUrl;

#[async_trait]
pub trait Repository {
    async fn list_pull_requests(&self) -> Result<()>;
    async fn create_pull_request(
        &self,
        source_branch: &str,
        target_branch: &str,
        title: &str,
    ) -> Result<()>;
}

pub fn get_remote_url() -> Result<GitUrl> {
    let remote_url = spawn("git remote get-url origin")?;
    Ok(GitUrl::parse(remote_url.trim())?)
}

pub async fn get_repo() -> Result<Box<dyn Repository>> {
    let remote_url: GitUrl = get_remote_url().expect("no remote is set for current repository");
    let remote_host = remote_url
        .clone()
        .host
        .ok_or(Error::msg("cannot resolve host of remote url"))?;

    let repo = match remote_host.as_ref() {
        "github.com" => bail!("WIP: unsupported repo type"),
        "gitlab.com" => bail!("WIP: unsupported repo type"),

        _ => GitLabRepository::init(&remote_host, &remote_url).await?,
    };

    Ok(Box::new(repo))
}
