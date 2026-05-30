use crate::types::{PortalUrl, Source};

pub async fn detect() -> Vec<PortalUrl> {
    detect_inner().await.unwrap_or_default()
}

async fn detect_inner() -> Result<Vec<PortalUrl>, Box<dyn std::error::Error + Send + Sync>> {
    // Try all common lease file paths across distros
    let candidates = [
        "/var/lib/dhcp/dhclient.leases",        // Debian/Ubuntu
        "/var/lib/dhclient/dhclient.leases",    // RHEL/Fedora/CentOS
        "/var/lib/dhclient/dhclient6.leases",
        "/var/lib/NetworkManager/dhclient.conf", // NM managed
    ];

    let mut results = Vec::new();
    for path in &candidates {
        if let Ok(content) = tokio::fs::read_to_string(path).await {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("option dhcp-captive-portal-uri") {
                    if let Some(url) = extract_quoted(trimmed) {
                        if url::Url::parse(url).is_ok() {
                            results.push(PortalUrl {
                                url: url.to_owned(),
                                source: Source::DhcpLease,
                                confidence: 95,
                            });
                        }
                    }
                }
            }
        }
    }

    // De-duplicate, keep last occurrence (most recent lease)
    results.dedup_by(|a, b| a.url == b.url);
    Ok(results)
}

fn extract_quoted(line: &str) -> Option<&str> {
    let start = line.find('"')? + 1;
    let end = line.rfind('"')?;
    if start < end { Some(&line[start..end]) } else { None }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_quoted_uri() {
        let line = r#"  option dhcp-captive-portal-uri "http://192.168.1.1/portal";"#;
        assert_eq!(extract_quoted(line), Some("http://192.168.1.1/portal"));
    }

    #[test]
    fn empty_string_returns_none() {
        assert_eq!(extract_quoted("option dhcp-captive-portal-uri"), None);
    }
}
