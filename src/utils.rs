use anyhow::{bail, Result};
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use std::{
    process::Command,
    ffi::OsStr,
    io::{stdin, stdout, Write},
};

const FRAGMENT: &AsciiSet = &CONTROLS.add(b'/');

pub fn url_encode(component: &str) -> String {
    utf8_percent_encode(component, FRAGMENT).to_string()
}

pub fn spawn(command: &str) -> Result<String> {
    let mut parts = command.split(' ');
    let program = parts.next().unwrap();
    let args: Vec<&OsStr> = parts.map(OsStr::new).collect();
    let mut cmd = Command::new(program);

    cmd.args(args);

    if !cmd.status()?.success() {
        bail!(format!("Failed to execute {}", command))
    }

    let buf = cmd.output()?.stdout;

    let result = String::from_utf8(buf)?;

    Ok(result)
}

pub fn get_current_branch() -> Result<String> {
    spawn("git rev-parse --abbrev-ref HEAD").map(|x| x.trim().to_string())
}

pub fn get_latest_commit_message() -> Result<String> {
    spawn("git rev-list --format=%B --max-count=1 HEAD").map(|x| {
        x.trim()
            .split("\n")
            .skip(1)
            .collect::<Vec<&str>>()
            .join("\n")
            .to_string()
    })
}

pub fn get_git_config(key: &str) -> Result<String> {
    spawn(&format!("git config --get {}", key)).map(|x| x.trim().to_string())
}

pub fn user_input(prompt: &str) -> Result<String> {
    stdout().write_all(prompt.as_bytes())?;
    stdout().flush()?;
    let mut input = String::new();
    stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn() {
        assert_eq!(spawn("echo 123").ok().unwrap(), "123\n")
    }

    #[test]
    fn test_get_latest_commit_message() {
        println!("result=[{}]", get_latest_commit_message().ok().unwrap())
    }
}
