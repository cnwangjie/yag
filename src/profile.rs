use anyhow::Result;
use log::debug;
use serde_derive::*;
use std::env;
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GitLabSelfHostedConfig {
    host: String,
    token: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Profile {
    gitlab_self_hosted: Option<GitLabSelfHostedConfig>,
}

impl Profile {
    fn new() -> Profile {
        Profile {
            gitlab_self_hosted: None,
        }
    }

    pub fn get_token_by_host(&self, host: &str) -> Option<String> {
        let gitlab_self_hosted = self.gitlab_self_hosted.as_ref()?;
        if gitlab_self_hosted.host.eq(host) {
            Some(gitlab_self_hosted.token.clone())
        } else {
            None
        }
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
    let profile = match profile_exists() {
        true => {
            let data = fs::read_to_string(get_profile_path())?;
            toml::from_str::<Profile>(&data)?
        }
        false => {
            let profile = Profile::new();
            write_profile(&profile).await?;
            profile
        }
    };
    debug!("loaded profile: {:#?}", profile);
    Ok(profile)
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;

    #[test]
    fn test_profile() {
        println!("{:#?}", block_on(load_profile()).ok().unwrap());
    }
}
