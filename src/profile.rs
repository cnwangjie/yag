use serde_derive::*;
use std::env;
use std::path::Path;
use std::fs;
use anyhow::Result;
use super::*;

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
      Some(gitlab_self_hosted.host.clone())
    } else {
      None
    }
  }
}

fn get_profile_path() -> String {
  env::var("HOME").unwrap_or("".to_string()) + "/.yag/profile"
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
  if !profile_exists() {
    let profile = Profile::new();
    write_profile(&profile).await?;
    Ok(profile)
  } else {
    let data = fs::read_to_string(get_profile_path())?;
    let profile: Profile = toml::from_str(&data)?;

    Ok(profile)
  }
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
