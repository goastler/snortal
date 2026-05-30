use crate::types::{PortalUrl, Source};

pub async fn detect() -> Vec<PortalUrl> {
    detect_inner().await.unwrap_or_default()
}

async fn detect_inner() -> Result<Vec<PortalUrl>, Box<dyn std::error::Error + Send + Sync>> {
    let mut results = Vec::new();

    // NM writes one file per device index (typically 1–16)
    for i in 1u8..=16 {
        let path = format!("/run/NetworkManager/devices/{i}");
        if let Ok(content) = tokio::fs::read_to_string(&path).await {
            if let Some(url) = parse_nm_device_file(&content) {
                results.push(PortalUrl {
                    url,
                    source: Source::NetworkManagerDevice,
                    confidence: 90,
                });
            }
        }
    }

    Ok(results)
}

/// Parse INI-style NM device file, return captive-portal-uri from [dhcp4] section if present.
fn parse_nm_device_file(content: &str) -> Option<String> {
    let mut in_dhcp4 = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_dhcp4 = trimmed == "[dhcp4]";
            continue;
        }
        if in_dhcp4 {
            if let Some(rest) = trimmed.strip_prefix("captive-portal-uri=") {
                let url = rest.trim().trim_matches('"');
                if url::Url::parse(url).is_ok() {
                    return Some(url.to_owned());
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_portal_uri_in_dhcp4_section() {
        let content = "[device]\ninterface=eth0\n[dhcp4]\nip_address=10.0.0.5\ncaptive-portal-uri=http://10.0.0.1/portal\n[dhcp6]\n";
        assert_eq!(
            parse_nm_device_file(content),
            Some("http://10.0.0.1/portal".to_owned())
        );
    }

    #[test]
    fn ignores_uri_outside_dhcp4_section() {
        let content = "[device]\ncaptive-portal-uri=http://ignored.example/\n[dhcp4]\nip_address=10.0.0.5\n";
        assert_eq!(parse_nm_device_file(content), None);
    }
}
