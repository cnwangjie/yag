use anyhow::Result;
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use std::ffi::OsStr;
use std::process::Command;

const FRAGMENT: &AsciiSet = &CONTROLS.add(b'/');

pub fn url_encode(component: &str) -> String {
    utf8_percent_encode(component, FRAGMENT).to_string()
}

pub fn spawn(command: &str) -> Result<String> {
    let mut parts = command.split(' ');
    let program = parts.next().unwrap();
    let args: Vec<&OsStr> = parts.map(OsStr::new).collect();

    let buf = Command::new(program).args(args).output()?.stdout;

    let result = String::from_utf8(buf)?;

    Ok(result)
}

pub fn get_current_branch() -> Result<String> {
    spawn("git branch --show-current")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn() {
        assert_eq!(spawn("echo 123").ok().unwrap(), "123\n")
    }
}
