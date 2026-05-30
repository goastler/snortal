pub mod dhcp_lease;
pub mod gateway_probe;
pub mod http_probe;
pub mod networkmanager_device;
pub mod nm_connectivity_conf;
pub mod nmcli;
pub mod wpa_supplicant;

use crate::types::{PortalUrl, StatusNote};
use futures::future::join_all;

pub struct DetectorResults {
    pub urls: Vec<PortalUrl>,
    pub notes: Vec<StatusNote>,
}

pub async fn run_all(timeout_secs: u64, verbose: bool) -> DetectorResults {
    let url_futures = vec![
        tokio::spawn(dhcp_lease::detect()),
        tokio::spawn(networkmanager_device::detect()),
        tokio::spawn(http_probe::detect(timeout_secs)),
        tokio::spawn(nm_connectivity_conf::detect()),
        tokio::spawn(gateway_probe::detect(timeout_secs)),
    ];

    let urls: Vec<PortalUrl> = join_all(url_futures)
        .await
        .into_iter()
        .filter_map(|r| r.ok())
        .flatten()
        .collect();

    let nmcli_notes = tokio::spawn(nmcli::detect(timeout_secs)).await.unwrap_or_default();

    if verbose {
        tokio::spawn(wpa_supplicant::detect(timeout_secs))
            .await
            .ok();
    }

    DetectorResults { urls, notes: nmcli_notes }
}
