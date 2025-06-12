use std::fmt::Debug;

pub trait IntoCommand<Args> {
    fn into_command(self) -> Command;
}

impl<H> IntoCommand<()> for H
where
    H: CommandHandler<()> + Send + Sync + 'static,
{
    fn into_command(self) -> Command {
        Command::NoArgs(Box::new(self))
    }
}

impl<H> IntoCommand<String> for H
where
    H: CommandHandler<String> + Send + Sync + 'static,
{
    fn into_command(self) -> Command {
        Command::OneArg(Box::new(self))
    }
}

#[async_trait::async_trait]
pub trait CommandHandler<Args> {
    async fn call(&self, args: Args) -> String;
}

#[async_trait::async_trait]
impl<F, Fut, Res> CommandHandler<()> for F
where
    F: Fn() -> Fut + Sync,
    Fut: Future<Output = Res> + Send,
    Res: ToString,
{
    async fn call(&self, _args: ()) -> String {
        self().await.to_string()
    }
}

#[async_trait::async_trait]
impl<F, Fut, Res> CommandHandler<String> for F
where
    F: Fn(String) -> Fut + Sync,
    Fut: Future<Output = Res> + Send,
    Res: ToString,
{
    async fn call(&self, args: String) -> String {
        self(args).await.to_string()
    }
}

pub enum Command {
    NoArgs(Box<dyn CommandHandler<()> + Send + Sync>),
    OneArg(Box<dyn CommandHandler<String> + Send + Sync>),
}

impl Debug for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoArgs(_) => f
                .debug_tuple("NoArgs")
                .field(&"<CommandHandler<()> -> String")
                .finish(),
            Self::OneArg(_) => f
                .debug_tuple("OneArg")
                .field(&"<CommandHandler<String> -> String")
                .finish(),
        }
    }
}
