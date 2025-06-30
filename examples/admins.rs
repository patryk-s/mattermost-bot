use anyhow::{Context, Result};
use mattermost_bot::MattermostBot;

#[tokio::main]
async fn main() -> Result<()> {
    let mybot = MattermostBot::new()?
        .add_command("help", help)
        .add_command("status", status)
        .add_admin_command("list", list)
        .add_admin("@bob".to_string());
    tracing::info!("Starting bot");
    mybot.listen().await.context("problem listening")
}

async fn help() -> &'static str {
    "Available commands:
  - status: show status
  - list NUM: get list item number"
}

async fn status() -> &'static str {
    "The status is OK"
}

async fn list(index: String) -> String {
    let index: i64 = match index.parse() {
        Ok(n) => n,
        Err(e) => return format!("cannot parse {index} to a number: {e}"),
    };
    let stuff = ["one", "two", "three"];
    if index >= stuff.len() as i64 || index < 0 {
        return format!(
            "Error: number out of bounds. Available range: `0 -- {}`",
            stuff.len() - 1
        );
    }
    format!("{index}. element is {}", stuff[index as usize])
}
