use std::collections::HashMap;
use std::env;
use std::fmt::Debug;

use mattermost_api::client::{AuthenticationData, Mattermost};
use tokio::runtime;
use tracing::trace;

mod command;
mod error;
mod handler;

use command::IntoCommand;
use handler::Handler;

pub use self::error::{Error, Result};

pub struct MattermostBot {
    handler: Handler,
    client: Mattermost,
}

impl MattermostBot {
    /// Create a `MattermostBot` using env variables for url and token.
    pub fn new() -> Result<Self> {
        let commands = HashMap::new();
        let mm_url =
            env::var("MATTERMOST_URL").map_err(|_| Error::EnvVarMissing("MATTERMOST_URL"))?;
        let mm_token =
            env::var("MATTERMOST_TOKEN").map_err(|_| Error::EnvVarMissing("MATTERMOST_TOKEN"))?;
        let auth_data = AuthenticationData::from_access_token(mm_token);
        let client = Mattermost::new(&mm_url, auth_data).map_err(Error::MattermostApi)?;
        let listener = client.clone();
        Ok(MattermostBot {
            handler: Handler { commands, client },
            client: listener,
        })
    }

    #[must_use]
    pub fn add_command<H, Args>(mut self, name: &str, handler: H) -> Self
    where
        H: IntoCommand<Args>,
    {
        self.handler
            .commands
            .insert(name.into(), handler.into_command());
        trace!("adding command: {name}");
        self
    }

    pub fn listen(self) -> Result<()> {
        let rt = runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let handler = self.handler;
        let mut listener = self.client;
        rt.block_on(listener.connect_to_websocket(handler))
            .map_err(Error::MattermostApi)
    }
}

impl Default for MattermostBot {
    /// Create a `MattermostBot` using env variables for url and token
    ///
    /// Panics
    /// Will panic if an env is missing
    fn default() -> Self {
        match Self::new() {
            Ok(bot) => bot,
            Err(e) => panic!("Failed to create bot: {e:?}"),
        }
    }
}

impl Debug for MattermostBot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MattermostBot")
            .field("commands", &self.handler.commands.keys())
            .finish_non_exhaustive()
    }
}
