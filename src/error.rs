pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Please set the '{0}' environment variable")]
    EnvVarMissing(&'static str),
    #[error("problem with mattermost api")]
    MattermostApi(#[source] mattermost_api::errors::ApiError),
}
