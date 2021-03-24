use crate::repository::Repository;

use super::client::GitHubClient;
use super::profile::GitHubConfig;
use super::structs::{GitHubResponse, Pull, SearchResult};
use crate::profile::load_profile;
use crate::structs::{PaginationResult, PullRequest};
use anyhow::{anyhow, bail, Result};
use async_trait::async_trait;
use git_url_parse::GitUrl;
use log::debug;
use reqwest::Method;
use serde_json::json;

pub struct GitHubRepository {
    repo: String,
    client: GitHubClient,
}

impl GitHubRepository {
    pub async fn init(remote_url: &GitUrl) -> Result<Self> {
        let profile = load_profile().await?;
        let config = profile
            .github
            .ok_or(anyhow!("no GitHub profile: Try `yag profile add` first"))?;
        let client = match config {
            GitHubConfig {
                access_token: Some(token),
                token: _,
                username: _,
            } => GitHubClient::build_with_oauth_token(&token)?,
            GitHubConfig {
                access_token: _,
                token: Some(token),
                username: Some(username),
            } => GitHubClient::build_with_basic_auth(&username, &token)?,
            _ => bail!("wrong GitHub profile config"),
        };
        Ok(GitHubRepository {
            repo: remote_url.fullname.to_string(),
            client: client,
        })
    }
}

#[async_trait]
impl Repository for GitHubRepository {
    async fn get_pull_request(&self, id: usize) -> Result<PullRequest> {
        let res = self
            .client
            .call(Method::GET, &format!("/repos/{}/pulls/{}", self.repo, id))
            .send()
            .await?;

        let text = res.text().await?;
        debug!("res: {}", text);

        serde_json::from_str::<GitHubResponse<Pull>>(&text)?
            .map(|pr| Ok(PullRequest::from(pr.to_owned())))
    }

    async fn list_pull_requests(
        &self,
        opt: crate::repository::ListPullRequestOpt,
    ) -> Result<PaginationResult<PullRequest>> {
        debug!("opt: {:#?}", opt);

        let mut pairs: Vec<(String, String)> =
            vec![("is".into(), "pr".into()), ("is".into(), "open".into())];

        pairs.push(("repo".into(), self.repo.to_owned()));

        if opt.me {
            pairs.push(("author".into(), "@me".into()));
        } else if let Some(author) = opt.author.clone() {
            pairs.push(("author".into(), author));
        }

        let res = self
            .client
            .call(Method::GET, "/search/issues")
            .query(&[("per_page", "10")])
            .query(&[("page", opt.get_page())])
            .query(&[("q", self.build_query(&pairs))])
            .send()
            .await?;

        let text = res.text().await?;
        debug!("res: {}", text);
        let result = serde_json::from_str::<GitHubResponse<SearchResult<Pull>>>(&text)?;
        result.map::<PaginationResult<PullRequest>, _>(|r| Ok(PaginationResult::from(r.clone())))
    }

    async fn create_pull_request(
        &self,
        source_branch: &str,
        target_branch: &str,
        title: &str,
    ) -> Result<PullRequest> {
        let res = self
            .client
            .call(Method::POST, &format!("/repos/{}/pulls", self.repo))
            .body(
                json!({
                    "title": title,
                    "head": source_branch,
                    "base": target_branch,
                })
                .to_string(),
            )
            .send()
            .await?;

        let text = res.text().await?;
        debug!("res: {}", text);

        serde_json::from_str::<GitHubResponse<Pull>>(&text)?
            .map(|pr| Ok(PullRequest::from(pr.to_owned())))
    }

    async fn close_pull_request(&self, id: usize) -> Result<PullRequest> {
        let res = self
            .client
            .call(Method::PATCH, &format!("/repos/{}/pulls/{}", self.repo, id))
            .body(
                json!({
                    "state": "closed",
                })
                .to_string(),
            )
            .send()
            .await?;

        let text = res.text().await?;
        debug!("{:#?}", text);

        serde_json::from_str::<GitHubResponse<Pull>>(&text)?
            .map(|data| Ok(PullRequest::from(data.to_owned())))
    }
}

impl GitHubRepository {
    fn build_query(&self, pairs: &[(String, String)]) -> String {
        pairs
            .iter()
            .map(|(k, v)| format!("{}:{}", k, v))
            .collect::<Vec<String>>()
            .join(" ")
    }
}
