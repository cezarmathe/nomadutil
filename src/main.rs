#[deny(missing_docs)]
#[macro_use]
extern crate rust_embed;

mod artifacts;
mod checkpoint;
mod cmd;
mod common;
mod install;
mod releases;
mod security;

use chrono::Local;

use clap::App;
use clap::Arg;
use clap::SubCommand;

use colored::*;

use log::Level;
use log::LevelFilter;

const NOMADUTIL_VERSION: &str = "0.1.0";

#[cfg(target_arch = "x86_64")]
pub const ARCH: &str = "amd64";

#[cfg(target_os = "linux")]
fn main() {
    let install = SubCommand::with_name("install")
        .about("Install Nomad.")
        .arg(Arg::with_name("version").long("version").takes_value(true).help(
            "The version of Nomad to install. If omitted, the latest version shall be used.",
        ))
        .arg(Arg::with_name("skip-sums").long("skip-sums").help(
            "Skip checking the sha256sums on the zip archive.",
        ))
        .arg(Arg::with_name("skip-sig").long("skip-sig").help(
            "Skip checking the signature of the sha256sums file. This has no effect if --skip-sums is used.",
        ))
        .arg(Arg::with_name("out").short("o").long("out").takes_value(true).help(
            "Where to place the nomad binary.",
        ))
        .arg(Arg::with_name("service-out").long("service-out").takes_value(true).help(
            "Where to place the nomad systemd service file.",
        ))
        .arg(Arg::with_name("ignore-alerts").long("ignore-alerts").help(
            "Ignore alerts for a version, if there are any alerts.",
        ))
        .arg(Arg::with_name("ignore-outdated").long("ignore-outdated").help(
            "Ignore whether a version is outdated.",
        ));

    let uninstall = SubCommand::with_name("uninstall").about("Uninstall Nomad.");

    let upgrade = SubCommand::with_name("upgrade")
        .about("Upgrade Nomad.")
        .arg(Arg::with_name("version")
            .long("version")
            .help("The version of Nomad to upgrade to. If omitted, Nomad will be upgraded to the latest version."));

    let start = SubCommand::with_name("start").about("Start(and enable) the Nomad service.");

    let stop = SubCommand::with_name("stop").about("Stop(and disable) the Nomad service.");

    let restart = SubCommand::with_name("restart").about("Restart the Nomad service.");

    let app: App = App::new("nomadutil")
        .version(NOMADUTIL_VERSION)
        .author("Armand Cezar Mathe <me@cezarmathe.com>")
        .about("Utility for managing Nomad.")
        .arg(
            Arg::with_name("verbosity")
                .short("v")
                .multiple(true)
                .help("Set the verbosity level of the messages outputed by eri. (-v for debug level, -vv for trace level)"),
        )
        .subcommand(install)
        .subcommand(uninstall)
        .subcommand(upgrade)
        .subcommand(start)
        .subcommand(stop)
        .subcommand(restart)
        .subcommand(SubCommand::with_name("info")
            .about("Get information about nomadutil."));

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

    match matches.subcommand() {
        ("info", Some(_)) => {
            println!("nomadutil {} {}", NOMADUTIL_VERSION, ARCH);
        }
        ("install", Some(args)) => {
            let opts = install::InstallOpts::from(args);

            if let Err(e) = install::install_do(args.value_of("version"), opts.into()) {
                log::error!("failed to install: {}", e);
                std::process::exit(1);
            }
        }

        _ => log::error!("Run nomadutil help."),
    }

    log::info!("done!");
}
