//! SSL Certificate Watcher Module
//!
//! Monitors SSL certificate files for changes and validates new certificates.
//! When certificates are renewed, this module detects the change and validates
//! that the new certificates are valid before they're used.
//!
//! # Architecture
//!
//! Due to ntex's architecture, SSL acceptors are bound at server startup.
//! This module provides certificate change detection and validation, logging
//! warnings when new certificates are available. For production deployments,
//! the recommended approach is to let systemd restart the service automatically
//! when certificate files change.
//!
//! # Usage
//!
//! ```rust
//! // Start watching certificates in background
//! cert_watcher::start_watching(
//!     cert_path.clone(),
//!     key_path.clone(),
//! );
//! ```

use anyhow::{Context, Result};
use notify::{Event, RecursiveMode, Watcher};
use openssl::x509::X509;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

/// Information about a certificate file
#[derive(Debug, Clone)]
pub struct CertInfo {
    pub path: PathBuf,
    pub modified: SystemTime,
    pub valid_until: Option<SystemTime>,
}

impl CertInfo {
    /// Load certificate information from a file
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let metadata = std::fs::metadata(path)
            .with_context(|| format!("Failed to read metadata for {}", path.display()))?;

        let modified = metadata
            .modified()
            .context("Failed to get modification time")?;

        // Try to parse the certificate to get expiration date
        let valid_until = Self::get_cert_expiration(path).ok();

        Ok(Self {
            path: path.to_path_buf(),
            modified,
            valid_until,
        })
    }

    /// Extract certificate expiration date
    fn get_cert_expiration(path: &Path) -> Result<SystemTime> {
        let cert_pem = std::fs::read(path)
            .with_context(|| format!("Failed to read certificate from {}", path.display()))?;

        let cert = X509::from_pem(&cert_pem)
            .context("Failed to parse X509 certificate")?;

        let not_after = cert.not_after();
        let epoch = SystemTime::UNIX_EPOCH;

        // Convert ASN1 time to SystemTime (approximate)
        let days_since_epoch = not_after.diff_days(&openssl::asn1::Asn1Time::from_unix(0).unwrap())
            .context("Failed to calculate certificate expiration")?;

        Ok(epoch + std::time::Duration::from_secs(days_since_epoch as u64 * 86400))
    }

    /// Check if certificate has been modified since this info was created
    pub fn has_changed(&self) -> Result<bool> {
        let current = Self::load(&self.path)?;
        Ok(current.modified != self.modified)
    }

    /// Validate that both certificate and key files are valid
    pub fn validate_cert_pair(cert_path: &Path, key_path: &Path) -> Result<()> {
        // Check files exist
        if !cert_path.exists() {
            anyhow::bail!("Certificate file does not exist: {}", cert_path.display());
        }
        if !key_path.exists() {
            anyhow::bail!("Private key file does not exist: {}", key_path.display());
        }

        // Try to load certificate
        let cert_pem = std::fs::read(cert_path)
            .with_context(|| format!("Failed to read certificate from {}", cert_path.display()))?;

        X509::from_pem(&cert_pem)
            .context("Failed to parse X509 certificate - file may be corrupted")?;

        // Try to load private key
        let key_pem = std::fs::read(key_path)
            .with_context(|| format!("Failed to read private key from {}", key_path.display()))?;

        openssl::pkey::PKey::private_key_from_pem(&key_pem)
            .context("Failed to parse private key - file may be corrupted")?;

        log::info!("✓ Certificate validation successful");
        Ok(())
    }
}

/// Start watching certificate files for changes
///
/// This function spawns a background task that monitors the certificate and key files.
/// When changes are detected, it validates the new certificates and logs appropriate messages.
///
/// # Arguments
/// * `cert_path` - Path to the SSL certificate file
/// * `key_path` - Path to the SSL private key file
///
/// # Note
/// This function returns immediately after starting the watcher.
/// The watcher runs in a background thread.
pub fn start_watching(cert_path: String, key_path: String) {
    tokio::spawn(async move {
        if let Err(e) = watch_certificates_impl(cert_path, key_path).await {
            log::error!("Certificate watcher error: {}", e);
        }
    });
}

/// Internal implementation of certificate watching
async fn watch_certificates_impl(cert_path: String, key_path: String) -> Result<()> {
    log::info!("🔍 Starting certificate file watcher");
    log::info!("   Monitoring: {}", cert_path);
    log::info!("   Monitoring: {}", key_path);

    let cert_path = Arc::new(PathBuf::from(cert_path));
    let key_path = Arc::new(PathBuf::from(key_path));

    // Get parent directories to watch
    let cert_dir = cert_path
        .parent()
        .context("Certificate file has no parent directory")?
        .to_path_buf();

    let key_dir = key_path
        .parent()
        .context("Private key file has no parent directory")?
        .to_path_buf();

    // Load initial certificate info
    let initial_cert = CertInfo::load(cert_path.as_ref())?;
    if let Some(valid_until) = initial_cert.valid_until {
        log::info!("📅 Current certificate valid until: {:?}", valid_until);
    }

    // Create channel for file system events
    let (tx, mut rx) = tokio::sync::mpsc::channel(100);

    // Create file watcher
    let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        if let Ok(event) = res {
            let _ = tx.blocking_send(event);
        }
    })?;

    // Watch both directories
    watcher.watch(&cert_dir, RecursiveMode::NonRecursive)?;
    if cert_dir != key_dir {
        watcher.watch(&key_dir, RecursiveMode::NonRecursive)?;
    }

    log::info!("✓ Certificate watcher started successfully");

    // Process file system events
    while let Some(event) = rx.recv().await {
        // Check if the event affects our certificate files
        let affects_cert = event.paths.iter().any(|p| p == cert_path.as_ref());
        let affects_key = event.paths.iter().any(|p| p == key_path.as_ref());

        if affects_cert || affects_key {
            log::info!("🔄 Detected certificate file change");

            // Wait a bit for file write to complete
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

            // Validate new certificates
            match CertInfo::validate_cert_pair(&cert_path, &key_path) {
                Ok(_) => {
                    log::warn!("🔐 New SSL certificates detected and validated!");
                    log::warn!("⚠️  Server restart required to use new certificates");
                    log::warn!("   Systemd will automatically restart the service if configured");

                    // Load new cert info to log expiration
                    if let Ok(new_cert) = CertInfo::load(cert_path.as_ref()) {
                        if let Some(valid_until) = new_cert.valid_until {
                            log::info!("📅 New certificate valid until: {:?}", valid_until);
                        }
                    }
                }
                Err(e) => {
                    log::error!("❌ New certificate validation failed: {}", e);
                    log::error!("   Continuing to use existing certificates");
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cert_validation() {
        // This test requires actual certificate files
        // In production, you would test with test certificates
    }
}
