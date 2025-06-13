use std::collections::HashMap;

use mattermost_api::{
    client::Mattermost,
    models::{Post, PostBody},
    socket::{WebsocketEvent, WebsocketEventType, WebsocketHandler},
};
use serde::Deserialize;
use tracing::{debug, error, trace};

use crate::command::Command;
use crate::{Error, Result};

pub(crate) struct Handler {
    pub(crate) admins: Vec<String>,
    pub(crate) admin_commands: HashMap<String, Command>,
    pub(crate) commands: HashMap<String, Command>,
    pub(crate) client: Mattermost,
}

impl Handler {
    async fn run_command(&self, user: &String, name: &str, arg: Option<String>) -> String {
        if !self.commands.contains_key(name) && !self.admin_commands.contains_key(name) {
            return format!("Unsupported command '{name}'");
        }
        // Check if the user called a normal or admin command.
        // If it's an admin command, check if the user is an admin.
        let command = match self.commands.get(name) {
            Some(normal_command) => normal_command,
            None => match self.admin_commands.get(name) {
                Some(admin_command) if self.admins.contains(user) => admin_command,
                _ => return "You are not allowed to run this command.".into(),
            },
        };
        match command {
            Command::NoArgs(handler) => handler.call(()).await,
            Command::OneArg(handler) => handler.call(arg.unwrap_or_default()).await,
        }
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
        let posted_data: PostedData = match serde_json::from_value(message.data) {
            Ok(v) => v,
            Err(e) => {
                error!("Problem decoding message data: {e}");
                return;
            }
        };
        // ignore all messages not directed to bot
        if posted_data.mentions.as_ref().is_none_or(|m| m.is_empty()) {
            return;
        }
        trace!("{:#?}", posted_data);
        let post: Post = match serde_json::from_str(&posted_data.post) {
            Ok(v) => v,
            Err(e) => {
                error!("Problem decoding post: {e}");
                return;
            }
        };

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
        let output = self
            .run_command(&posted_data.sender_name, command_name, Some(args))
            .await;
        let channel_id = &message.broadcast.channel_id;
        self.post_reply(channel_id, &post.id, &output)
            .await
            .expect("post_reply failed");
    }
}

// define `data` type for `Posted` `WebsocketEventType`
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct PostedData {
    channel_display_name: String,
    channel_name: String,
    channel_type: String,
    mentions: Option<String>,
    post: String,
    sender_name: String,
    set_online: bool,
    team_id: String,
}
