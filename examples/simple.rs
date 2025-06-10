use anyhow::{Context, Result};
use mattermost_bot::MattermostBot;

fn main() -> Result<()> {
    let mybot = MattermostBot::new()?
        .add_command("help", help)
        .add_command("status", status)
        .add_command("list", list);
    mybot.listen().context("problem listening")
}

fn help() -> String {
    "Available commands:
  - status: show status
  - list: get list"
        .into()
}

fn status() -> String {
    "The status is OK".into()
}

fn list() -> String {
    let stuff = vec!["one", "two", "three"];
    format!("{stuff:?}")
}
