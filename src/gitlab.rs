use std::fmt;
use crate::{profile::load_profile, repository::ListPullRequestOpt};
use crate::repository::Repository;
use crate::structs::{PullRequest, PaginationResult};
use crate::utils::url_encode;
use anyhow::*;
use async_trait::async_trait;
use git_url_parse::GitUrl;
use log::debug;
use reqwest::header::HeaderMap;
use reqwest::{Client, Method, RequestBuilder, Url};
use serde_derive::*;
use serde_json::{json, Value};

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
        message: Option<Value>,
    },
}

impl<T> GitLabResponse<T> where T: fmt::Debug {
    #[inline]
    fn map<R, F>(&self, f: F) -> Result<R> where F: FnOnce(&T) -> Result<R> {
        match self {
            GitLabResponse::Data(data) => f(data),
            GitLabResponse::Error { error, message } => {
                debug!("found an error: {:#?}", self);

                let message = message
                    .clone()
                    .and_then(|message| {
                        match message {
                            Value::String(message) => Some(message),
                            Value::Array(messages) => {
                                let message = messages
                                    .iter()
                                    .filter_map(|i| i.as_str())
                                    .map(|i| i.to_string())
                                    .collect::<Vec<String>>()
                                    .join("\n");
                                Some(message)
                            },
                            _ => None,
                        }
                    })
                    .or_else(move || error.clone().map(|i| i.join("\n")))
                    .unwrap_or("unknown error".to_string());

                bail!(message)
            },
        }
    }
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
