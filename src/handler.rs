use std::collections::HashMap;

use mattermost_api::{
    client::Mattermost,
    models::{Post, PostBody},
    socket::{WebsocketEvent, WebsocketEventType, WebsocketHandler},
};
use tracing::{debug, trace};

use crate::command::Command;
use crate::{Error, Result};

pub(crate) struct Handler {
    pub(crate) commands: HashMap<String, Command>,
    pub(crate) client: Mattermost,
}

impl Handler {
    async fn run_command(&self, name: &str, arg: Option<String>) -> String {
        if !self.commands.contains_key(name) {
            return format!("unsupported command '{name}'");
        }
        self.commands
            .get(name)
            .map(async |c| match c {
                Command::NoArgs(handler) => handler.call(()).await,
                Command::OneArg(handler) => handler.call(arg.unwrap_or_default()).await,
            })
            .unwrap()
            .await
    }

    /// Post a message to the `channel_id` as a reply to the post with `root_id`
    pub async fn post_reply(&self, channel_id: &str, root_id: &str, message: &str) -> Result<()> {
        let _ = self
            .client
            .create_post(&PostBody {
                channel_id: channel_id.into(),
                message: message.into(),
                root_id: Some(root_id.into()),
            })
            .await
            .map_err(Error::MattermostApi)?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl WebsocketHandler for Handler {
    async fn callback(&self, message: WebsocketEvent) {
        if message.event == WebsocketEventType::Hello {
            trace!("Got hello event");
            return;
        }
        if message.event != WebsocketEventType::Posted {
            trace!("Skipping event: {:?}", message.event);
            return;
        }
        // ignore all messages not directed to bot
        if message.data.get("mentions").is_none() {
            return;
        }
        trace!("{:#?}", message.data);
        let post: Post = serde_json::from_str(
            message
                .data
                .get("post")
                .expect("post missing")
                .as_str()
                .expect("post not a string type"),
        )
        .expect("error deserializing post");

        let mut words = post.message.split_whitespace();
        let first = words.next().expect("missing at least one word");
        let command_name = if first.starts_with('@') {
            words.next().unwrap_or("missing command name")
        } else {
            first
        };
        debug!("got command {command_name}");
        let remainder: Vec<&str> = words.collect();
        let args = remainder.join(" ");
        let output = self.run_command(command_name, Some(args)).await;
        let channel_id = &message.broadcast.channel_id;
        self.post_reply(channel_id, &post.id, &output)
            .await
            .expect("post_reply failed");
    }
}
