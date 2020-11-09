//! Module for nomadutil commands.

use clap::App;
use clap::ArgMatches;

mod info;
mod install;

pub use info::InfoCmd;
pub use install::InstallCmd;

/// Register subcommands
#[macro_export]
macro_rules! register_subcommands {
    ($app:ident, commands: { $($cmdname:ty),* }) => {
        $(
            $app = <$cmdname>::register($app);
        )*
    };
}

/// Macro for matching on subcommands and running them.
#[macro_export]
macro_rules! match_subcommands {
    ($matches: ident, commands: { $($cmdname:ty),* }) => {
        match $matches.subcommand() {
            $(
                (<$cmdname>::NAME, Some(args)) => {
                    let _cmd = <$cmdname>::new(args);

                    if let Err(e) = _cmd.run() {
                        log::error!("{} failed: {}", <$cmdname>::NAME, e);
                        std::process::exit(1);
                    }
                },
            )*
            _ => {
                log::error!("You should probably run $ nomadutil help.");
                std::process::exit(1);
            },
        };
        log::info!("done!");
    };
}

/// Trait for defining common command behaviour.
pub trait Command {
    /// The name of this command.
    const NAME: &'static str;

    /// Create a new command.
    fn new(args: &ArgMatches) -> Self;
    //  Register this command in the clap app.
    fn register(app: App<'static, 'static>) -> App<'static, 'static>;
    /// Run this command.
    fn run(&self) -> anyhow::Result<()>;
}
