use crate::types::{PortalUrl, Source};
use futures::future::join_all;
use reqwest::redirect;
use std::time::Duration;

// Known connectivity-check endpoints used by major OS/browser vendors.
// On a captive portal these will be redirected to the login page.
const DEFAULT_PROBES: &[&str] = &[
    "http://connectivity-check.ubuntu.com./",
    "http://captive.apple.com/",
    "http://www.gstatic.com/generate_204",
    "http://detectportal.firefox.com/success.txt",
    "http://nmcheck.gnome.org/check_network_status.txt",
    "http://network-test.debian.org/nm",
];

/// `probes` overrides the built-in endpoint list when non-empty.
pub async fn detect(timeout_secs: u64, probes: &[String]) -> Vec<PortalUrl> {
    detect_inner(timeout_secs, probes).await.unwrap_or_default()
}

async fn detect_inner(
    timeout_secs: u64,
    probes: &[String],
) -> Result<Vec<PortalUrl>, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::builder()
        .redirect(redirect::Policy::none())
        .timeout(Duration::from_secs(timeout_secs))
        .build()?;

    let endpoints: Vec<String> = if probes.is_empty() {
        DEFAULT_PROBES.iter().map(|s| s.to_string()).collect()
    } else {
        probes.to_vec()
    };

    let futures: Vec<_> = endpoints
        .into_iter()
        .map(|endpoint| probe_one(client.clone(), endpoint))
        .collect();

    Ok(join_all(futures).await.into_iter().flatten().collect())
}

async fn probe_one(client: reqwest::Client, endpoint: String) -> Option<PortalUrl> {
    let response = client.get(&endpoint).send().await.ok()?;

    if response.status().is_redirection() {
        let location = response
            .headers()
            .get(reqwest::header::LOCATION)?
            .to_str()
            .ok()?
            .to_owned();

        // Resolve relative redirects against the probe URL
        let resolved = if location.starts_with("http://") || location.starts_with("https://") {
            location
        } else {
            let base = url::Url::parse(&endpoint).ok()?;
            base.join(&location).ok()?.to_string()
        };

        if url::Url::parse(&resolved).is_ok() {
            return Some(PortalUrl {
                url: resolved,
                source: Source::HttpProbe { endpoint },
                confidence: 85,
            });
        }
    }

    None
}
