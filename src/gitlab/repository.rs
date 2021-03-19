use crate::{profile::load_profile, repository::ListPullRequestOpt};
use crate::repository::Repository;
use crate::structs::{PullRequest, PaginationResult};
use anyhow::*;
use async_trait::async_trait;
use super::structs::User;
use super::client::GitLabClient;
use git_url_parse::GitUrl;
use log::debug;
use reqwest::Method;
use serde_json::json;
use super::structs::{GitLabResponse, MergeRequest};

pub struct GitLabRepository {
  client: GitLabClient,
  project_id: u64,
}

impl GitLabRepository {
  pub async fn init(host: &str, remote_url: &GitUrl) -> Result<Self> {
      let profile = load_profile().await?;
      let token = profile
          .get_token_by_host(host)
          .expect(&format!("unknown remote host: {}", host));
      let client = GitLabClient::build(host, &token)?;
      let project_id = client.get_project_id(remote_url.fullname.as_ref()).await?;
      Ok(GitLabRepository {
          client,
          project_id,
      })
  }
}

#[async_trait]
impl Repository for GitLabRepository {
  async fn get_pull_request(&self, id: usize) -> Result<PullRequest> {
      let res = self.client
          .call(Method::GET, &format!("/api/v4/projects/{}/merge_requests/{}", self.project_id, id))
          .send()
          .await?;

      let text = res.text().await?;

      debug!("{:#?}", text);

      serde_json::from_str::<GitLabResponse<MergeRequest>>(&text)?
          .map(|data| Ok(PullRequest::from(data.to_owned())))
  }

  async fn list_pull_requests(&self, opt: ListPullRequestOpt) -> Result<PaginationResult<PullRequest>> {
      let mut req = self
          .client
          .call(
              Method::GET,
              &format!("/api/v4/projects/{}/merge_requests", self.project_id),
          )
          .query(&[("state", "opened"), ("per_page", "10")])
          .query(&[("page", opt.get_page())]);

      if let Some(username) = opt.author {
          let user = self.get_user_by_username(&username).await?;
          req = req.query(&[("author_id", user.id)]);
      }

      if opt.me {
          req = req.query(&[("scope", "created-by-me")]);
      }

      let res = req
          .send()
          .await?;

      debug!("{:#?}", res);

      let total = res.headers()
          .get("x-total")
          .map(|v| v.to_str().ok())
          .flatten()
          .map(|v| v.parse::<u64>().ok())
          .flatten()
          .ok_or(anyhow!("fail to get total"))?;

      let text = res.text().await?;
      debug!("{:#?}", text);

      let result = serde_json::from_str::<GitLabResponse<Vec<MergeRequest>>>(&text)?
          .map(|mr| {
              Ok(mr.iter()
                  .map(|mr| mr.to_owned())
                  .map(PullRequest::from)
                  .collect())
          })?;

      Ok(PaginationResult::new(result, total))
  }

  async fn create_pull_request(
      &self,
      source_branch: &str,
      target_branch: &str,
      title: &str,
  ) -> Result<PullRequest> {
      let res = self
          .client
          .call(
              Method::POST,
              &format!("/api/v4/projects/{}/merge_requests", self.project_id),
          )
          .header("Content-Type", "application/json")
          .body(
              json!({
                "source_branch": source_branch,
                "target_branch": target_branch,
                "title": title,
              })
              .to_string(),
          )
          .send()
          .await?;

      let text = res.text().await?;

      debug!("{:#?}", text);

      serde_json::from_str::<GitLabResponse<MergeRequest>>(&text)?
          .map(|data| Ok(PullRequest::from(data.to_owned())))
  }

  async fn close_pull_request(&self, id: usize) -> Result<PullRequest> {
      let res = self.client
          .call(Method::PUT, &format!("/api/v4/projects/{}/merge_requests/{}", self.project_id, id))
          .header("Content-Type", "application/json")
          .body(
              json!({
                  "state_event": "close",
              }).to_string(),
          )
          .send()
          .await?;

      let text = res.text().await?;
      debug!("{:#?}", text);

      serde_json::from_str::<GitLabResponse<MergeRequest>>(&text)?
          .map(|data| Ok(PullRequest::from(data.to_owned())))
  }
}

impl GitLabRepository {
  async fn get_user_by_username(&self, username: &str) -> Result<User> {
      let res = self.client
          .call(Method::GET, &format!("/api/users"))
          .query(&[("username", username)])
          .send()
          .await?;

      let text = res.text().await?;

      serde_json::from_str::<GitLabResponse<Vec<User>>>(&text)?
          .map(|data| {
              data.first().cloned().ok_or(anyhow!("unexpected empty response"))
          })
  }
}
