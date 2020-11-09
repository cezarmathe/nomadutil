//! Install Nomad.

use crate::checkpoint::check;
use crate::common::opt_string_to_opt_str;
use crate::releases::*;

use super::Command;

use std::fs::OpenOptions;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::path::PathBuf;

use clap::App;
use clap::Arg;
use clap::ArgMatches;
use clap::SubCommand;

/// Default output dir for the nomad binary.
const DEFAULT_NOMAD_OUT: &str = "/usr/local/bin";
/// Default output dir for the nomad service file.
const DEFAULT_NOMAD_SERVICE_OUT: &str = "/etc/systemd/system";

/// Install command.
pub struct InstallCmd {
    version: Option<String>,
    /// check the integrity of the zip archive
    check_integrity: bool,
    /// check the signature of the shasums
    check_sig: bool,
    /// where to install the nomad binary
    out: PathBuf,
    /// where to install the nomad service file
    service_out: PathBuf,
    /// whether to ignore alerts or not
    ignore_alerts: bool,
    /// whether to ignore if a version is outdated or not
    ignore_outdated: bool,
}

impl Command for InstallCmd {
    const NAME: &'static str = "install";

    fn new(args: &ArgMatches) -> Self {
        Self {
            version: if let Some(value) = args.value_of("version") {
                Some(value.to_string())
            } else {
                None
            },
            check_integrity: !args.is_present("skip-sums"),
            check_sig: !args.is_present("skip-sig"),
            out: if let Some(value) = args.value_of("out") {
                value.into()
            } else {
                PathBuf::from(DEFAULT_NOMAD_OUT)
            },
            service_out: if let Some(value) = args.value_of("service-out") {
                value.into()
            } else {
                PathBuf::from(DEFAULT_NOMAD_SERVICE_OUT)
            },
            ignore_alerts: args.is_present("ignore-alerts"),
            ignore_outdated: args.is_present("ignore-outdated"),
        }
    }

    fn register(app: App<'static, 'static>) -> App<'static, 'static> {
        let install = SubCommand::with_name(Self::NAME)
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
        app.subcommand(install)
    }

    fn run(&self) -> anyhow::Result<()> {
        let version = opt_string_to_opt_str(&self.version);
        let res = check(version)?;
        let version: &str = if let Some(value) = version {
            value
        } else {
            res.current_version()
        };
        if res.outdated() {
            if self.ignore_outdated {
                log::warn!(
                    "checkpoint says version {} is outdated, newest is {}, ignoring",
                    version,
                    res.current_version()
                );
            } else {
                anyhow::bail!(
                    "checkpoint says version {} is outdated, newest is {}",
                    version,
                    res.current_version()
                );
            }
        } else {
            log::info!("{} is the latest release", version);
        }
        if !res.alerts().is_empty() {
            if self.ignore_alerts {
                log::warn!("alerts: {:?}; ignoring", res.alerts());
            } else {
                anyhow::bail!("alerts: {:?}", res.alerts());
            }
        }

        log::info!("attempting to install version {}", version);

        let bin = get(version, Some(ReleaseGetOpts::from(self)))?;
        log::info!("nomad binary ready for installation");

        let out = {
            let mut out = if !self.out.is_absolute() {
                self.out.canonicalize()?
            } else {
                self.out.clone()
            };
            if out.is_dir() {
                out.push("nomad");
            }
            out
        };

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .mode(0o755)
            .open(&out)?;
        let written = file.write(bin.as_ref())?;
        if written != bin.len() {
            anyhow::bail!(
                "nomad binary: written {} bytes instead, not {}",
                written,
                bin.len()
            );
        }

        log::info!("nomad binary installed");

        let service_file_contents = format!(
            "[Unit]
Description=Nomad
Documentation=https://nomadproject.io/docs/
Wants=network-online.target
After=network-online.target

[Service]
ExecReload=/bin/kill -HUP $MAINPID
ExecStart={} agent -config /etc/nomad.d
KillMode=process
KillSignal=SIGINT
LimitNOFILE=infinity
LimitNPROC=infinity
Restart=on-failure
RestartSec=2
StartLimitBurst=3
StartLimitIntervalSec=10
TasksMax=infinity

[Install]
WantedBy=multi-user.target",
            &out.display()
        );

        let service_out = {
            let mut service_out = if !self.service_out.is_absolute() {
                self.service_out.canonicalize()?
            } else {
                self.service_out.clone()
            };
            if service_out.is_dir() {
                service_out.push("nomad.service");
            }
            service_out
        };

        let mut service_file = OpenOptions::new()
            .write(true)
            .create(true)
            .mode(0o644)
            .open(&service_out)?;
        let service_written = service_file.write(service_file_contents.as_bytes())?;
        if service_written != service_file_contents.len() {
            anyhow::bail!(
                "nomad service file: written {} bytes instead, not {}",
                service_written,
                service_file_contents.len()
            );
        }

        log::info!("nomad service file installed");

        Ok(())
    }
}

impl InstallCmd {
    #[allow(missing_docs, dead_code)]
    #[inline]
    pub fn check_integrity(&self) -> bool {
        self.check_integrity
    }

    #[allow(missing_docs, dead_code)]
    #[inline]
    pub fn check_sig(&self) -> bool {
        self.check_sig
    }

    #[allow(missing_docs, dead_code)]
    #[inline]
    pub fn out(&self) -> &Path {
        self.out.as_path()
    }

    #[allow(missing_docs, dead_code)]
    #[inline]
    pub fn service_out(&self) -> &Path {
        self.service_out.as_path()
    }

    #[allow(missing_docs, dead_code)]
    #[inline]
    pub fn ignore_alerts(&self) -> bool {
        self.ignore_alerts
    }

    #[allow(missing_docs, dead_code)]
    #[inline]
    pub fn ignore_outdated(&self) -> bool {
        self.ignore_outdated
    }
}
