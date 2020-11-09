//! Module for getting a release.

use crate::artifacts::*;
use crate::cmd::InstallCmd;
use crate::security::*;

use std::io::Cursor;
use std::io::Read;

use bytes::Bytes;

use zip::ZipArchive;

/// Options for getting a release.
pub struct ReleaseGetOpts {
    /// check the integrity of the zip archive
    check_integrity: bool,
    /// check the signature of the shasums
    check_sig: bool,
}

impl Default for ReleaseGetOpts {
    #[allow(missing_docs)]
    fn default() -> Self {
        Self::new(true, true)
    }
}

impl From<&InstallCmd> for ReleaseGetOpts {
    #[allow(missing_docs)]
    fn from(src: &InstallCmd) -> Self {
        Self {
            check_integrity: src.check_integrity(),
            check_sig: src.check_sig(),
        }
    }
}

impl ReleaseGetOpts {
    /// Create a new ReleaseGetOpts.
    #[inline]
    pub fn new(check_integrity: bool, check_sig: bool) -> Self {
        Self {
            check_integrity,
            check_sig,
        }
    }

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
}

/// Get a Nomad release.
///
/// This will return the nomad binary after it has been verifief for integrity and uncompressed.
pub fn get(version: &str, opts: Option<ReleaseGetOpts>) -> anyhow::Result<Bytes> {
    let opts: ReleaseGetOpts = if let Some(value) = opts {
        value
    } else {
        ReleaseGetOpts::default()
    };

    let sums: Option<Sha256Sums> = if opts.check_integrity {
        let sums = Sha256Sums::get(version)?;
        log::info!("downloaded checksums for version {}", version);

        if !opts.check_sig {
            log::warn!("not checking the signature of the shasums");
        } else {
            let sig = Sha256SumsSig::get(version)?;
            log::info!("downloaded checksums signature for version {}", version);

            let sig_checker = SigChecker::new()?;
            let _ = sig_checker.check(sig.inner(), sums.inner())?;
            log::info!("checksums signature ok");
        }

        Some(sums)
    } else {
        None
    };

    let buf = {
        let zip = {
            let zip = NomadZip::get(version)?;
            log::info!("downloaded nomad zip archive for version {}", version);

            if !opts.check_integrity {
                log::warn!("not checking the integrity of the zip archive")
            } else {
                let sums_checker = SumsChecker::new(sums.unwrap().inner(), version)?;
                let _ = sums_checker.check(zip.inner())?;
                log::info!("zip archive ok");
            }

            zip
        };
        let mut zip = ZipArchive::new(Cursor::new(zip.inner()))?;

        if zip.is_empty() {
            anyhow::bail!("empty archive");
        }
        if zip.len() != 1 {
            log::warn!(
                "zip archive: {} files: {:?}",
                zip.len(),
                zip.file_names().collect::<Vec<&str>>()
            );
            anyhow::bail!("malformed zip archive");
        }

        let mut zip_file = zip.by_name("nomad")?;
        let mut buf: Vec<u8> = Vec::with_capacity(zip_file.size() as usize);
        let _ = zip_file.read_to_end(&mut buf)?;
        buf
    };

    log::info!("unzipped the nomad artifact");

    Ok(Bytes::from(buf))
}
