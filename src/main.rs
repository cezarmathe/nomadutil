#[macro_use]
extern crate rust_embed;

mod artifacts;
mod checkpoint;
#[macro_use]
mod cmd;
mod common;
mod releases;
mod security;

use cmd::*;

use chrono::Local;

use clap::App;
use clap::AppSettings;
use clap::Arg;

use colored::*;

use log::Level;
use log::LevelFilter;

const NOMADUTIL_VERSION: &str = "0.1.0";

#[cfg(target_arch = "x86_64")]
pub const ARCH: &str = "amd64";

#[cfg(target_arch = "aarch64")]
pub const ARCH: &str = "arm64";

#[cfg(target_os = "linux")]
fn main() {
    let app: App = {
        let mut app = App::new("nomadutil")
        .version(NOMADUTIL_VERSION)
        .author("Armand Cezar Mathe <me@cezarmathe.com>")
        .about("Utility for managing Nomad.")
        .setting(AppSettings::AllowExternalSubcommands)
        .arg(
            Arg::with_name("verbosity")
                .short("v")
                .multiple(true)
                .help("Set the verbosity level of the messages outputed by eri. (-v for debug level, -vv for trace level)"),
        );

        register_subcommands!(app, commands: {
            InfoCmd,
            InstallCmd
        });

        app
    };

    let matches = app.get_matches();

    let log_level: LevelFilter = match &matches.occurrences_of("verbosity") {
        0 => LevelFilter::Info,
        1 => LevelFilter::Debug,
        2 => LevelFilter::Trace,
        _ => {
            println!("The source code is available at https://github.com/cezarmathe/nomadutil.");
            std::process::exit(0);
        }
    };
    fern::Dispatch::new()
        .format(|out, message, record| {
            let prefix: String = match record.level() {
                Level::Error => "ERROR >".red().bold().to_string(),
                Level::Warn => "WARN  >".yellow().bold().to_string(),
                Level::Info => "INFO  >".blue().bold().to_string(),
                Level::Debug => "DEBUG >".cyan().bold().to_string(),
                Level::Trace => "TRACE >".purple().bold().to_string(),
            };
            let time = Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();
            out.finish(format_args!("{} {} {}", time, prefix, message));
        })
        .level(log_level)
        .chain(std::io::stdout())
        .apply()
        .unwrap();

    log::trace!("nomadutil ready");

    match_subcommands!(matches, commands: {
        InfoCmd,
        InstallCmd
    });
}
