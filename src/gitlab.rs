use std::fmt;
use crate::{profile::load_profile, repository::ListPullRequestOpt};
use crate::repository::Repository;
use crate::structs::PullRequest;
use crate::utils::url_encode;
use anyhow::*;
use async_trait::async_trait;
use git_url_parse::GitUrl;
use log::debug;
use reqwest::header::HeaderMap;
use reqwest::{Client, Method, RequestBuilder, Url};
use serde_derive::*;
use serde_json::json;

#[derive(Deserialize, Serialize, Debug)]
pub struct Project {
    id: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub username: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MergeRequest {
    id: u64,
    iid: u64,
    project_id: u64,
    title: String,
    description: Option<String>,
    state: String,
    created_at: String,
    updated_at: String,
    target_branch: String,
    source_branch: String,
    author: User,
    web_url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum GitLabResponse<T> {
    Data(T),
    Error {
        error: Option<Vec<String>>,
        message: Option<serde_json::Value>,
    },
}

impl<T> fmt::Display for GitLabResponse<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        match &*self {
            GitLabResponse::Error {
                error,
                message,
            } => {
                let msg = message.as_ref().and_then(|m| Some(format!("{:#}", m)))
                    .or_else(|| error.as_ref().and_then(|err| Some(err.join("\n"))))
                    .unwrap_or("unknown error".to_string());
                write!(f, "{}", msg)
            },
            _ => write!(f, "[data]")
        }
    }
}

impl From<MergeRequest> for PullRequest {
    fn from(mr: MergeRequest) -> Self {
        Self {
            id: mr.iid,
            title: mr.title,
            author: mr.author.username,
            base: mr.target_branch,
            head: mr.source_branch,
            updated_at: mr.updated_at,
            url: mr.web_url,
        }
    }
}

pub struct GitLabClient {
    host: String,
    client: reqwest::Client,
}

impl GitLabClient {
    fn build(host: &str, token: &str) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert("Private-Token", token.parse()?);
        debug!("default headers: {:?}", headers);
        let client = Client::builder().default_headers(headers).build()?;

        Ok(GitLabClient {
            host: host.to_string(),
            client: client,
        })
    }

    fn call(&self, method: Method, uri: &str) -> RequestBuilder {
        let mut url = Url::parse(&format!("https://{}", self.host)).unwrap();
        url.set_path(uri);

        self.client.request(method, url)
    }

    async fn get_project_id(&self, name: &str) -> Result<u64> {
        let res = self
            .call(
                Method::GET,
                &format!("/api/v4/projects/{}", url_encode(name)),
            )
            .send()
            .await?;

        let data = res.error_for_status()?.json::<Project>().await?;

        Ok(data.id)
    }
}

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
    // TODO: add error handling for all methods
    async fn get_pull_request(&self, id: usize) -> Result<PullRequest> {
        let res = self.client
            .call(Method::GET, &format!("/api/v4/projects/{}/merge_requests/{}", self.project_id, id))
            .send()
            .await?;
        let mr = res.json::<MergeRequest>().await?;

        Ok(PullRequest::from(mr))
    }

    async fn list_pull_requests(&self, opt: ListPullRequestOpt) -> Result<Vec<PullRequest>> {
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
            req = req.query(&[("scope", "me")]);
        }

        let res = req
            .send()
            .await?;

        debug!("{:#?}", res);

        let data: Vec<MergeRequest> = res.json::<Vec<MergeRequest>>().await?;

        debug!("{:#?}", data);

        let pull_requests = data.iter()
            .map(|mr| mr.to_owned())
            .map(PullRequest::from)
            .collect();

        Ok(pull_requests)
    }

    async fn create_pull_request(
        &self,
        source_branch: &str,
        target_branch: &str,
        title: &str,
    ) -> Result<PullRequest> {
        debug!("source: {} target: {}", source_branch, target_branch);
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

        let data = serde_json::from_str::<GitLabResponse<MergeRequest>>(&text)?;

        debug!("{:#?}", data);

        match data {
            GitLabResponse::Data(data) => Ok(PullRequest::from(data)),
            _ => bail!(format!("{}", data)),
        }
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

        let data = serde_json::from_str::<GitLabResponse<MergeRequest>>(&text)?;

        debug!("{:#?}", data);
        match data {
            GitLabResponse::Data(data) => Ok(PullRequest::from(data)),
            _ => bail!(format!("{}", data)),
        }
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

        let data = serde_json::from_str::<GitLabResponse<Vec<User>>>(&text)?;

        match data {
            GitLabResponse::Data(data) => data.first().cloned().ok_or(anyhow!("unexpected empty response")),
            _ => bail!(format!("{}", data),)
        }
    }
}
