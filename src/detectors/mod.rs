pub mod dhcp_lease;
pub mod gateway_probe;
pub mod http_probe;
pub mod networkmanager_device;
pub mod nm_connectivity_conf;
pub mod nmcli;
pub mod wpa_supplicant;

use crate::types::{PortalUrl, StatusNote};
use futures::future::join_all;

/// Runtime overrides for individual detectors.
#[derive(Debug, Default)]
pub struct DetectorConfig {
    pub timeout_secs: u64,
    pub verbose: bool,
    /// Replaces built-in HTTP probe endpoints when non-empty.
    pub probe_endpoints: Vec<String>,
    /// Replaces built-in DHCP lease file paths when non-empty.
    pub dhcp_files: Vec<String>,
    /// Overrides auto-detected gateway IP for the gateway HTTP probe.
    pub gateway_ip: Option<String>,
}

pub struct DetectorResults {
    pub urls: Vec<PortalUrl>,
    pub notes: Vec<StatusNote>,
}

pub async fn run_all(cfg: &DetectorConfig) -> DetectorResults {
    let dhcp_files = cfg.dhcp_files.clone();
    let probe_endpoints = cfg.probe_endpoints.clone();
    let gateway_ip = cfg.gateway_ip.clone();
    let timeout = cfg.timeout_secs;

    let url_futures = vec![
        tokio::spawn(async move { dhcp_lease::detect(&dhcp_files).await }),
        tokio::spawn(networkmanager_device::detect()),
        tokio::spawn(async move { http_probe::detect(timeout, &probe_endpoints).await }),
        tokio::spawn(nm_connectivity_conf::detect()),
        tokio::spawn(async move {
            gateway_probe::detect(timeout, gateway_ip.as_deref()).await
        }),
    ];

    let urls: Vec<PortalUrl> = join_all(url_futures)
        .await
        .into_iter()
        .filter_map(|r| r.ok())
        .flatten()
        .collect();

    let nmcli_notes = tokio::spawn(nmcli::detect(timeout)).await.unwrap_or_default();

    if cfg.verbose {
        tokio::spawn(wpa_supplicant::detect(timeout)).await.ok();
    }

    DetectorResults { urls, notes: nmcli_notes }
}
