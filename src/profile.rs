use anyhow::{Result, anyhow};
use log::debug;
use serde_derive::*;
use std::{borrow::{Borrow, BorrowMut}, env};
use std::fs;
use std::path::Path;
use crate::utils;

pub trait ProfileConfig {
    fn fill_profile(&self, profile: &mut Profile);
}

pub trait Prompter {
    fn display_name(&self) -> String;
    fn prompt(&self) -> Result<Box<dyn ProfileConfig>>;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GitLabSelfHostedConfig {
    host: String,
    token: String,
}

impl ProfileConfig for GitLabSelfHostedConfig {
    fn fill_profile(&self, profile: &mut Profile) {
        let mut default = vec![];
        let configs = profile.gitlab_self_hosted.as_mut().unwrap_or(&mut default);
        configs.push(self.to_owned());
    }
}

#[derive(Default)]
pub struct GitLabSelfHostedPrompter;

impl Prompter for GitLabSelfHostedPrompter {
    fn display_name(&self) -> String {
        "GitLab (self-hosted)".to_string()
    }
    fn prompt(&self) -> Result<Box<dyn ProfileConfig>> {
        Ok(Box::new(GitLabSelfHostedConfig {
            host: utils::user_input("host: ")?,
            token: utils::user_input("token: ")?,
        }))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Profile {
    gitlab_self_hosted: Option<Vec<GitLabSelfHostedConfig>>,
}

impl Profile {
    fn new() -> Profile {
        Profile {
            gitlab_self_hosted: None,
        }
    }

    pub fn get_token_by_host(&self, host: &str) -> Option<String> {
        self.gitlab_self_hosted.as_ref().and_then(|configs| {
            configs.iter().find(|config| {
                config.host.eq(host)
            }).and_then(|config| {
                Some(config.token.clone())
            })
        })
    }
}

fn get_profile_path() -> String {
    env::var("HOME").unwrap_or("".to_string()) + "/.yag/profile.toml"
}

fn profile_exists() -> bool {
    Path::new(&get_profile_path()).exists()
}

async fn write_profile(profile: &Profile) -> Result<()> {
    fs::create_dir_all(Path::new(&get_profile_path()).parent().unwrap())?;
    fs::write(get_profile_path(), toml::to_vec(profile)?)?;
    Ok(())
}

pub async fn load_profile() -> Result<Profile> {
    let profile = {
        if profile_exists() {
            let data = fs::read_to_string(get_profile_path())?;
            toml::from_str::<Profile>(&data)?
        } else {
            let profile = Profile::new();
            write_profile(&profile).await?;
            profile
        }
    };
    debug!("loaded profile: {:#?}", profile);
    Ok(profile)
}

pub async fn prompt_add_profile() -> Result<()> {
    let prompters: Vec<Box<dyn Prompter>> = vec![
        Box::new(GitLabSelfHostedPrompter::default()),
    ];
    for (i, prompter) in prompters.iter().enumerate() {
        println!("{:>3}: {}", i+1, prompter.display_name());
    }
    let choice_range = 1..=prompters.len();

    let profile_prompter = utils::user_input(&format!("select which type profile you want to create ({}-{}): ", choice_range.start(), choice_range.end()))?
        .parse::<usize>()
        .ok()
        .filter(|index| choice_range.contains(&index))
        .and_then(|index| prompters.get(index))
        .ok_or( anyhow!("invalid choice"))?;

    let profile_config = profile_prompter.prompt()?;
    let mut profile = load_profile().await?;
    profile_config.fill_profile(&mut profile);
    println!("{} profile added!", profile_prompter.display_name());
    Ok(())
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use futures::executor::block_on;

    use libc::STDIN_FILENO;
    use utils::user_input;

    use std::{fs::File, io::Write};
    use std::os::unix::io::{FromRawFd};

    #[tokio::test]
    async fn test_profile() -> Result<()> {
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
        prompt_add_profile().await?;
        Ok(())
    }
}
