use serde_derive::*;

use crate::profile::{Profile, ProfileConfig, Prompter};
use crate::utils;
use anyhow::{bail, Result};
use async_trait::async_trait;
use colored::*;

use super::client::GitHubAnonymousClient;
use super::structs::{AccessToken, GetAccessTokenResponse};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GitHubConfig {
    pub access_token: Option<String>,
    pub username: Option<String>,
    pub token: Option<String>,
}

impl ProfileConfig for GitHubConfig {
    fn fill_profile(&self, profile: &mut Profile) {
        profile.github = Some(self.to_owned());
    }
}
#[derive(Default)]
pub struct GitHubPrompter;

#[async_trait]
impl Prompter for GitHubPrompter {
    fn display_name(&self) -> String {
        "GitHub".to_string()
    }

    async fn prompt(&self) -> Result<Box<dyn ProfileConfig>> {
        let client = GitHubAnonymousClient::new()?;

        println!("Logging in GitHub...");
        let res = client.gen_device_code().await?;

        let notice = format!(
            "Please open {} in your browser and enter the code {}",
            res.verification_uri.bold(),
            res.user_code.green().bold(),
        );

        println!("{}", notice);
        utils::user_input("Press enter to continue")?;

        let mut re: Option<AccessToken> = None;

        while re.is_none() {
            let start_time = tokio::time::Instant::now();
            match client.gen_access_token(&res.device_code).await? {
                GetAccessTokenResponse::Ok(access_token) => re = Some(access_token),
                GetAccessTokenResponse::Error {
                    error,
                    error_description,
                    interval,
                } => {
                    match error.as_ref() {
                        "authorization_pending" => println!("{}", error_description),
                        "slow_down" => {
                            println!("{}", error_description);
                            let deadline =
                                start_time + core::time::Duration::from_secs(interval.unwrap_or(5));
                            println!(
                                "Please wait {} seconds",
                                deadline.duration_since(start_time).as_secs()
                            );

                            tokio::time::delay_until(deadline).await;
                        }
                        "expired_token" => bail!("expired"),
                        _ => bail!("unknown error"),
                    }
                    utils::user_input("Press enter to retry")?;
                }
            }
        }

        Ok(Box::new(GitHubConfig {
            access_token: Some(re.unwrap().access_token),
            username: None,
            token: None,
        }))
    }
}
