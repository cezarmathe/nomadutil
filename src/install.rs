//! Install Nomad.

use crate::checkpoint::check;
use crate::releases::*;

use std::fs::OpenOptions;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::path::PathBuf;

use clap::ArgMatches;

/// Default output dir for the nomad binary.
const DEFAULT_NOMAD_OUT: &str = "/usr/local/bin";
/// Default output dir for the nomad service file.
const DEFAULT_NOMAD_SERVICE_OUT: &str = "/etc/systemd/system";

pub struct InstallOpts {
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

impl InstallOpts {
    pub fn new<'a, B, P>(
        check_integrity: B,
        check_sig: B,
        out: P,
        service_out: P,
        ignore_alerts: B,
        ignore_outdated: B,
    ) -> Self
    where
        B: Into<Option<bool>>,
        P: Into<Option<&'a str>>,
    {
        Self::new_impl(
            check_integrity.into(),
            check_sig.into(),
            out.into(),
            service_out.into(),
            ignore_alerts.into(),
            ignore_outdated.into(),
        )
    }

    fn new_impl(
        check_integrity: Option<bool>,
        check_sig: Option<bool>,
        out: Option<&str>,
        service_out: Option<&str>,
        ignore_alerts: Option<bool>,
        ignore_outdated: Option<bool>,
    ) -> Self {
        Self {
            check_integrity: check_integrity.unwrap_or(true),
            check_sig: check_sig.unwrap_or(true),
            out: PathBuf::from(out.unwrap_or(DEFAULT_NOMAD_OUT)),
            service_out: PathBuf::from(service_out.unwrap_or(DEFAULT_NOMAD_SERVICE_OUT)),
            ignore_alerts: ignore_alerts.unwrap_or(false),
            ignore_outdated: ignore_outdated.unwrap_or(false),
        }
    }

    #[allow(missing_docs)]
    #[inline]
    pub fn check_integrity(&self) -> bool {
        self.check_integrity
    }

    #[allow(missing_docs)]
    #[inline]
    pub fn check_sig(&self) -> bool {
        self.check_sig
    }

    #[allow(missing_docs)]
    #[inline]
    pub fn out(&self) -> &Path {
        self.out.as_path()
    }

    #[allow(missing_docs)]
    #[inline]
    pub fn service_out(&self) -> &Path {
        self.service_out.as_path()
    }

    #[allow(missing_docs)]
    #[inline]
    pub fn ignore_alerts(&self) -> bool {
        self.ignore_alerts
    }

    #[allow(missing_docs)]
    #[inline]
    pub fn ignore_outdated(&self) -> bool {
        self.ignore_outdated
    }
}

impl Default for InstallOpts {
    #[allow(missing_docs)]
    fn default() -> Self {
        Self::new(None, None, None, None, None, None)
    }
}

impl<'a> From<&ArgMatches<'a>> for InstallOpts {
    fn from(args: &ArgMatches) -> Self {
        Self::new(
            !args.is_present("skip-sums"),
            !args.is_present("ski-sig"),
            if let Some(value) = args.value_of("out") {
                value.into()
            } else {
                None
            },
            if let Some(value) = args.value_of("service-out") {
                value.into()
            } else {
                None
            },
            args.is_present("ignore-alerts"),
            args.is_present("ignore-outdated"),
        )
    }
}

/// Install Nomad.
pub fn install_do(version: Option<&str>, opts: Option<InstallOpts>) -> anyhow::Result<()> {
    let opts = if let Some(value) = opts {
        value
    } else {
        InstallOpts::default()
    };

    let res = check(version)?;
    let version: &str = if let Some(value) = version {
        value
    } else {
        res.current_version()
    };
    if res.outdated() {
        if opts.ignore_outdated {
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
        if opts.ignore_alerts {
            log::warn!("alerts: {:?}; ignoring", res.alerts());
        } else {
            anyhow::bail!("alerts: {:?}", res.alerts());
        }
    }

    log::info!("attempting to install version {}", version);

    let bin = get(version, Some(ReleaseGetOpts::from(&opts)))?;
    log::info!("nomad binary ready for installation");

    let out = {
        let mut out = if !opts.out.is_absolute() {
            opts.out.canonicalize()?
        } else {
            opts.out.clone()
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
        let mut service_out = if !opts.service_out.is_absolute() {
            opts.service_out.canonicalize()?
        } else {
            opts.service_out.clone()
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
