//! Security-related module for checking sha25ssums and signature.

use std::borrow::Cow;
use std::io::Cursor;

use gpgrv::Keyring;

use sha2::Digest;
use sha2::Sha256;

/// Container for embedded assets.
///
/// Only used to embed the HashiCorp GPG key in the binary.
#[derive(RustEmbed)]
#[folder = "assets/"]
struct Assets;

/// Container for the components required to check signatures.
pub struct SigChecker {
    keyring: Keyring,
}

/// Container for the components required to check a zip archive's checksum.
pub struct SumsChecker {
    sums: Vec<u8>,
}

impl SigChecker {
    /// Create a new signature checker.
    pub fn new() -> anyhow::Result<Self> {
        let keyring = {
            let mut keyring = Keyring::new();

            let key: Cow<'static, [u8]> =
                if let Some(value) = Assets::get("security@hashicorp.com.key") {
                    value
                } else {
                    anyhow::bail!("failed to load the embedded gpg key");
                };
            let _ = keyring.append_keys_from_armoured(key.to_vec().as_slice());

            keyring
        };

        Ok(Self { keyring })
    }

    /// Check sums against a signature.
    pub fn check(&self, sig: &[u8], sums: &str) -> anyhow::Result<()> {
        let sums = Cursor::new(sums.as_bytes());
        gpgrv::verify_detached(sig, sums, &self.keyring)?;

        Ok(())
    }
}

impl SumsChecker {
    /// Create a new SumsChecker from a SHA256SUMS file.
    pub fn new(sums_raw: &str, version: &str) -> anyhow::Result<Self> {
        let mut sums_opt: Option<Vec<u8>> = None;
        for line in sums_raw.split('\n') {
            if line.len() == 0 {
                log::debug!("empty line, skipping");
                continue;
            }

            log::debug!("parsing sums: {}", line);

            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() != 2 {
                anyhow::bail!("malformed sums line: {}", line);
            }

            let artifact_fields: Vec<&str> =
                fields[1].trim_end_matches(".zip").split('_').collect();
            if artifact_fields.len() != 4 {
                anyhow::bail!("malformed sums artifact name: {}", fields[1]);
            }

            let ver = artifact_fields[1];
            let os = artifact_fields[2];
            let arch = artifact_fields[3];

            if os != "linux" {
                log::debug!("os {} is not linux", os);
                continue;
            }
            if arch != crate::ARCH {
                log::debug!("arch {} is not {}", arch, crate::ARCH);
                continue;
            }
            if ver != version {
                anyhow::bail!(
                    "malformed sums file: version {} in artifact name does not match version {}",
                    ver,
                    version
                );
            }

            log::debug!("found sums, stopping");
            sums_opt = Some(hex::decode(fields[0])?);
            break;
        }

        let sums = if let Some(value) = sums_opt {
            value
        } else {
            anyhow::bail!(
                "no sums found for version {} and arch {}",
                version,
                crate::ARCH
            );
        };

        Ok(Self { sums })
    }

    /// Check whether a file's digest matches the one provided.
    pub fn check(&self, src: &[u8]) -> anyhow::Result<()> {
        let sums = Sha256::digest(src);
        if sums.as_slice() != self.sums {
            anyhow::bail!(
                "artifact digest {} does not match provided digest {}",
                hex::encode(sums.as_slice()),
                hex::encode(self.sums.as_slice())
            );
        }
        Ok(())
    }
}
