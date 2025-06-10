use std::collections::HashMap;

use mattermost_api::{
    client::Mattermost,
    models::Post,
    socket::{WebsocketEvent, WebsocketEventType, WebsocketHandler},
};
use tracing::{debug, trace};

pub(crate) struct Handler {
    pub(crate) commands: HashMap<String, Box<dyn Fn() -> String + Send + Sync>>,
    pub(crate) client: Mattermost,
}

impl Handler {
    fn run_command(&self, name: &str) -> String {
        if !self.commands.contains_key(name) {
            return format!("unsupported command '{name}'");
        }
        let command = self.commands.get(name).unwrap();
        command()
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
        let output = self.run_command(command_name);
        let channel_id = &message.broadcast.channel_id;
        self.client
            .post_reply(channel_id, &post.id, &output)
            .await
            .expect("post_reply failed");
    }
}
