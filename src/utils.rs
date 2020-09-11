use anyhow::Result;
use std::ffi::OsStr;
use std::process::Command;
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};

const FRAGMENT: &AsciiSet = &CONTROLS.add(b'/');

pub fn url_encode(component: String) -> String {
    utf8_percent_encode(&component, FRAGMENT).to_string()
}

pub fn spawn(command: &str) -> Result<String> {
    let mut parts = command.split(' ');
    let program = parts.next().unwrap();
    let args: Vec<&OsStr> = parts.map(OsStr::new).collect();

    let buf = Command::new(program).args(args).output()?.stdout;

    let result = String::from_utf8(buf)?;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn() {
        assert_eq!(spawn("echo 123").ok().unwrap(), "123\n")
    }
}
