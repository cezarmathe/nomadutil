//! Module for getting the Nomad artifacts(zip, sums, sig).

use crate::common::get_http_client;

use bytes::Bytes;

use reqwest::header::ACCEPT;

/// Trait that defines common behaviour for remote artifacts that need to be downloaded.
pub trait RemoteArtifact<T> {
    /// Get the artifact.
    fn get(version: &str) -> anyhow::Result<T>;
}
/// Container for the Sha256Sums of the artifact.
#[derive(Clone, Debug)]
pub struct Sha256Sums {
    inner: String,
}

/// Container for the Sha256Sums signature.
#[derive(Clone, Debug)]
pub struct Sha256SumsSig {
    inner: Bytes,
}

/// Container for the zip archive that contains the nomad binary.
#[derive(Clone, Debug)]
pub struct NomadZip {
    inner: Bytes,
}

impl RemoteArtifact<Sha256Sums> for Sha256Sums {
    fn get(version: &str) -> anyhow::Result<Self> {
        let sums_res = get_http_client()
            .get(
                format!(
                    "https://releases.hashicorp.com/nomad/{0}/nomad_{0}_SHA256SUMS",
                    version
                )
                .as_str(),
            )
            .header(ACCEPT, "text/plain")
            .send()?;
        if !sums_res.status().is_success() {
            anyhow::bail!(
                "failed to get checksums for version {}: {}",
                version,
                sums_res.status()
            );
        }
        let sums = sums_res.text()?;

        Ok(Self { inner: sums })
    }
}

impl Sha256Sums {
    #[allow(missing_docs)]
    #[inline]
    pub fn inner(&self) -> &str {
        self.inner.as_str()
    }
}

impl RemoteArtifact<Sha256SumsSig> for Sha256SumsSig {
    fn get(version: &str) -> anyhow::Result<Self> {
        let sums_sig_res = get_http_client()
            .get(
                format!(
                    "https://releases.hashicorp.com/nomad/{0}/nomad_{0}_SHA256SUMS.sig",
                    version
                )
                .as_str(),
            )
            .header(ACCEPT, "application/octet-stream")
            .send()?;
        if !sums_sig_res.status().is_success() {
            anyhow::bail!(
                "failed to get checksums signature for version {}: {}",
                version,
                sums_sig_res.status()
            );
        }
        let sums_sig = sums_sig_res.bytes()?;

        Ok(Self { inner: sums_sig })
    }
}

impl Sha256SumsSig {
    #[allow(missing_docs)]
    #[inline]
    pub fn inner(&self) -> &[u8] {
        self.inner.as_ref()
    }
}

impl RemoteArtifact<NomadZip> for NomadZip {
    fn get(version: &str) -> anyhow::Result<Self> {
        let zip_res = get_http_client()
            .get(
                format!(
                    "https://releases.hashicorp.com/nomad/{0}/nomad_{0}_linux_{1}.zip",
                    version,
                    crate::ARCH
                )
                .as_str(),
            )
            .header(ACCEPT, "application/zip")
            .send()?;
        if !zip_res.status().is_success() {
            anyhow::bail!(
                "failed to get nomad zip archive for version {}: {}",
                version,
                zip_res.status()
            );
        }
        let zip = zip_res.bytes()?;

        Ok(Self { inner: zip })
    }
}

impl NomadZip {
    /// Get the inner value.
    #[inline]
    pub fn inner(&self) -> &[u8] {
        self.inner.as_ref()
    }
}
