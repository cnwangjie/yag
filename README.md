# yag
YAG is Yet Another Git CLI tool.

![Rust](https://github.com/cnwangjie/yag/workflows/Rust/badge.svg)
![crate.io](https://img.shields.io/crates/v/yag.svg)
![crate.io](https://img.shields.io/crates/d/yag.svg)

## Installation

You can just use following command to install if you have already have `cargo` in your environment.

```bash
cargo install yag
```


If you have not installed rust tools already you can try to install cargo with following command.

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

*above command is referenced from https://www.rust-lang.org/tools/install

## Example

```bash
# list all pull requests of current repository
yag pr list

# get the details of #3 pull request of current repository
yag pr get 3

# submit a new pull request from current branch to master
yag pr new
```

## Usage

Use `yag help` to see more details.

## License

under the MIT License
