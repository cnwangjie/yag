use crate::structs::{PaginationResult, PullRequest};
use crate::gitlab::GitLabRepository;
use crate::utils::spawn;
use anyhow::*;
use async_trait::async_trait;
use clap::ArgMatches;
use git_url_parse::GitUrl;

#[derive(Debug, Default)]
pub struct ListPullRequestOpt {
    pub author: Option<String>,
    page: Option<usize>,
    pub me: bool,
}

impl<'a> From<ArgMatches<'a>> for ListPullRequestOpt {
    fn from(matches: ArgMatches<'a>) -> Self {
        Self {
            author: matches.value_of("author").and_then(|s| Some(s.to_string())),
            page: matches.value_of("page").and_then(|s| s.parse::<usize>().ok()),
            me: matches.is_present("me"),
        }
    }
}

impl ListPullRequestOpt {
    pub fn get_page(&self) -> usize {
        self.page.unwrap_or(0)
    }
}

#[async_trait]
pub trait Repository {
    async fn get_pull_request(&self, id: usize) -> Result<PullRequest>;
    async fn list_pull_requests(&self, opt: ListPullRequestOpt) -> Result<PaginationResult<PullRequest>>;
    async fn create_pull_request(
        &self,
        source_branch: &str,
        target_branch: &str,
        title: &str,
    ) -> Result<PullRequest>;
    async fn close_pull_request(&self, id: usize) -> Result<PullRequest>;
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
