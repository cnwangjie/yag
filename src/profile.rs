use crate::github::profile::{GitHubConfig, GitHubPrompter};
use crate::gitlab::profile::{GitLabSelfHostedConfig, GitLabSelfHostedPrompter};
use crate::utils;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use log::debug;
use serde_derive::*;
use std::env;
use std::fs;
use std::path::Path;

pub trait ProfileConfig {
    fn fill_profile(&self, profile: &mut Profile);
}

#[async_trait]
pub trait Prompter {
    fn display_name(&self) -> String;
    async fn prompt(&self) -> Result<Box<dyn ProfileConfig>>;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Profile {
    pub gitlab_self_hosted: Option<Vec<GitLabSelfHostedConfig>>,
    pub github: Option<GitHubConfig>,
}

impl Profile {
    fn new() -> Self {
        Self {
            gitlab_self_hosted: None,
            github: None,
        }
    }

    pub fn get_gitlab_token_by_host(&self, host: &str) -> Option<String> {
        self.gitlab_self_hosted.as_ref().and_then(|configs| {
            configs
                .iter()
                .find(|config| config.host.eq(host))
                .and_then(|config| Some(config.token.clone()))
        })
    }
}

fn get_profile_path() -> String {
    env::var("HOME").unwrap_or("".to_string()) + "/.yag/profile.toml"
}

fn profile_exists() -> bool {
    Path::new(&get_profile_path()).exists()
}

pub async fn write_profile(profile: &Profile) -> Result<()> {
    fs::create_dir_all(Path::new(&get_profile_path()).parent().unwrap())?;
    fs::write(get_profile_path(), toml::to_vec(profile)?)?;
    Ok(())
}

fn migrate_profile(value: &mut toml::Value) {
    // gitlab_self_hosted map to array
    value.as_table_mut().map(|v| {
        if let Some(config) = v.get("gitlab_self_hosted") {
            if !config.is_array() {
                let config = config.clone();
                v.insert(
                    "gitlab_self_hosted".to_string(),
                    toml::Value::Array(vec![config]),
                );
            }
        }
    });
}

pub async fn load_profile() -> Result<Profile> {
    let profile = {
        if profile_exists() {
            let data = fs::read_to_string(get_profile_path())?;
            toml::from_str::<Profile>(&data).or_else(|_| {
                let mut value = toml::from_str::<toml::Value>(&data)?;
                debug!(
                    "failed to parse profile. attempt to migrate\nraw data: {:#?}",
                    value
                );
                migrate_profile(&mut value);
                value.try_into::<Profile>()
            })?
        } else {
            let profile = Profile::new();
            write_profile(&profile).await?;
            profile
        }
    };
    debug!("loaded profile: {:#?}", profile);
    Ok(profile)
}

pub async fn prompt_add_profile(profile: &mut Profile) -> Result<()> {
    let prompters: Vec<Box<dyn Prompter>> = vec![
        Box::new(GitLabSelfHostedPrompter::default()),
        Box::new(GitHubPrompter::default()),
    ];
    for (i, prompter) in prompters.iter().enumerate() {
        println!("{:>3}: {}", i + 1, prompter.display_name());
    }
    let choice_range = 1..=prompters.len();

    let profile_prompter = utils::user_input(&format!(
        "select which type profile you want to create ({}-{}): ",
        choice_range.start(),
        choice_range.end(),
    ))?
    .parse::<usize>()
    .ok()
    .filter(|index| choice_range.contains(&index))
    .and_then(|index| prompters.get(index - 1))
    .ok_or(anyhow!("invalid choice"))?;

    let profile_config = profile_prompter.prompt().await?;
    profile_config.fill_profile(profile);
    println!("{} profile added!", profile_prompter.display_name());
    Ok(())
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    use libc::STDIN_FILENO;
    use utils::user_input;

    use std::os::unix::io::FromRawFd;
    use std::{fs::File, io::Write};

    #[tokio::test]
    async fn test_profile() -> Result<()> {
        crate::logger::Logger::init(true)?;
        let profile = load_profile().await?;
        println!("{:#?}", profile);
        Ok(())
    }

    fn write_stdin(content: &str) -> Result<()> {
        let mut fds: [i32; 2] = [0; 2];
        unsafe { libc::pipe(fds.as_mut_ptr()) };
        let reader = fds[0];
        let original = unsafe { libc::dup(STDIN_FILENO) };
        unsafe { libc::dup2(reader, STDIN_FILENO) };
        let mut writer = unsafe { File::from_raw_fd(fds[1]) };
        writer.write_all(content.as_bytes())?;
        writer.flush()?;
        unsafe {
            libc::dup2(STDIN_FILENO, original);
            libc::close(fds[0]);
            libc::close(fds[1]);
        }
        Ok(())
    }

    #[test]
    fn test_write_stdin() -> Result<()> {
        write_stdin("111\n")?;
        let re = user_input("a:")?;
        assert_eq!(re, "111");
        Ok(())
    }

    #[tokio::test]
    async fn test_prompt_add_profile() -> Result<()> {
        let mut profile = Profile::new();
        write_stdin("1\n2\n3\n")?;
        prompt_add_profile(&mut profile).await?;
        assert!(profile.gitlab_self_hosted.is_some());
        let configs = profile.gitlab_self_hosted.unwrap();
        assert_eq!(configs.len(), 1);
        assert_eq!(configs[0].host, "2");
        assert_eq!(configs[0].token, "3");
        Ok(())
    }
}
