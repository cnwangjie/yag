use crate::github::repository::GitHubRepository;
use crate::gitlab::repository::GitLabRepository;
use crate::structs::{PaginationResult, PullRequest};
use crate::utils::spawn;
use anyhow::*;
use async_trait::async_trait;
use clap::ArgMatches;
use git_url_parse::GitUrl;
use log::debug;

#[derive(Debug, Default)]
pub struct ListPullRequestOpt {
    pub author: Option<String>,
    page: Option<usize>,
    pub me: bool,
    pub head: Option<String>,
}

impl<'a> From<ArgMatches<'a>> for ListPullRequestOpt {
    fn from(matches: ArgMatches<'a>) -> Self {
        debug!("matches: {:#?}", matches);
        Self {
            author: matches.value_of("author").and_then(|s| Some(s.to_string())),
            page: matches
                .value_of("page")
                .and_then(|s| s.parse::<usize>().ok()),
            me: matches.is_present("me"),
            head: None,
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
    async fn list_pull_requests(
        &self,
        opt: ListPullRequestOpt,
    ) -> Result<PaginationResult<PullRequest>>;
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
    let remote_url: GitUrl = get_remote_url()
        .ok()
        .ok_or(anyhow!("no remote is set for current repository"))?;

    let remote_host = remote_url
        .clone()
        .host
        .ok_or(Error::msg("cannot resolve host of remote url"))?;

    let repo: Box<dyn Repository> = match remote_host.as_ref() {
        "github.com" => Box::new(GitHubRepository::init(&remote_url).await?),
        "gitlab.com" => bail!("WIP: unsupported repo type"),
        _ => Box::new(GitLabRepository::init(&remote_host, &remote_url).await?),
    };

    Ok(repo)
}
