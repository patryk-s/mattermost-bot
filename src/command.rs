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

pub trait CommandHandler<Args> {
    fn call(&self, args: Args) -> String;
}

impl<F> CommandHandler<()> for F
where
    F: Fn() -> String,
{
    fn call(&self, _args: ()) -> String {
        self()
    }
}

impl<F> CommandHandler<String> for F
where
    F: Fn(String) -> String,
{
    fn call(&self, args: String) -> String {
        self(args)
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
