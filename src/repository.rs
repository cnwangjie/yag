use crate::profile::load_profile;
use crate::gitlab::GitLabRepository;
use crate::utils::spawn;
use git_url_parse::GitUrl;
use anyhow::*;

pub trait Repository {
  fn list_pull_requests(&self) -> Result<()>;
  fn create_pull_request(&self) -> Result<()>;
}

pub fn get_remote_url() -> Result<GitUrl> {
  let remote_url = spawn("git remote get-url origin")?;
  Ok(GitUrl::parse(remote_url.trim())?)
}

pub async fn get_repo() -> Result<Box<dyn Repository>> {
  let remote_url = get_remote_url()?;
  let remote_host = remote_url.host
    .ok_or(Error::msg("cannot resolve host of remote url"))?;

  match remote_host.as_ref() {
    "github.com" => bail!("WIP: unsupported repo type"),
    "gitlab.com" => bail!("WIP: unsupported repo type"),
    _ => Ok(Box::new(GitLabRepository {
      profile: load_profile().await?,
    })),
  }
}

