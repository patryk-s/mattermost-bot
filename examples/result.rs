use std::error::Error;

use mattermost_bot::MattermostBot;
use mattermost_bot::response::IntoResponse;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mybot = MattermostBot::new()?
        .add_command("help", help)
        .add_command("status", status)
        .add_command("list", list);
    Ok(mybot.listen().await?)
}

async fn help() -> &'static str {
    "Available commands:
  - status: show status
  - list NUM: get list item number"
}

async fn status() -> impl IntoResponse {
    "The status is OK"
}

async fn list(index: String) -> Result<String, MyError> {
    let index = index
        .parse::<i64>()
        .map_err(|e| MyError::InvalidInput(format!("cannot parse {index} to a number: {e}")))?;

    let stuff = ["one", "two", "three"];
    if index >= stuff.len() as i64 || index < 0 {
        return Err(MyError::InvalidInput(format!(
            "number out of bounds. Available range: `0 -- {}`",
            stuff.len() - 1,
        )));
    }
    Ok(format!("{index}. element is {}", stuff[index as usize]))
}

pub enum MyError {
    Sensitive(String),
    InvalidInput(String),
}

impl IntoResponse for MyError {
    fn into_response(self) -> String {
        match self {
            MyError::Sensitive(e) => {
                eprintln!("got error: {e:?}");
                "Something went wrong.".to_string()
            }
            MyError::InvalidInput(e) => format!("error from command: {e}"),
        }
    }
}
