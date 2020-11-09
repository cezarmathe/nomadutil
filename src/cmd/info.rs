//! Get the version of nomadutil.

use super::Command;

use clap::App;
use clap::ArgMatches;
use clap::SubCommand;

/// A command that shows the version of nomadutil.
pub struct InfoCmd {}

impl Command for InfoCmd {
    const NAME: &'static str = "info";

    fn new(_: &ArgMatches) -> Self {
        Self {}
    }

    fn register(app: App<'static, 'static>) -> App<'static, 'static> {
        let version = SubCommand::with_name(Self::NAME).about("Get info about nomadutil.");
        app.subcommand(version)
    }

    fn run(&self) -> anyhow::Result<()> {
        log::info!("nomadutil {}, {}", crate::NOMADUTIL_VERSION, crate::ARCH);
        Ok(())
    }
}
