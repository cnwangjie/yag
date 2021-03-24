use anyhow::Result;
use async_trait::async_trait;
use serde_derive::*;

use crate::{
    profile::{Profile, ProfileConfig, Prompter},
    utils,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GitLabSelfHostedConfig {
    pub host: String,
    pub token: String,
}

impl ProfileConfig for GitLabSelfHostedConfig {
    fn fill_profile(&self, profile: &mut Profile) {
        let mut default = vec![];
        let configs = profile.gitlab_self_hosted.as_mut().unwrap_or(&mut default);
        configs.push(self.to_owned());
        profile.gitlab_self_hosted = Some(configs.to_owned());
    }
}

#[derive(Default)]
pub struct GitLabSelfHostedPrompter;

#[async_trait]
impl Prompter for GitLabSelfHostedPrompter {
    fn display_name(&self) -> String {
        "GitLab (self-hosted)".to_string()
    }
    async fn prompt(&self) -> Result<Box<dyn ProfileConfig>> {
        Ok(Box::new(GitLabSelfHostedConfig {
            host: utils::user_input("host: ")?,
            token: utils::user_input("token: ")?,
        }))
    }
}
