use crate::types::{PortalUrl, Source};

pub async fn detect() -> Vec<PortalUrl> {
    detect_inner().await.unwrap_or_default()
}

async fn detect_inner() -> Result<Vec<PortalUrl>, Box<dyn std::error::Error + Send + Sync>> {
    // Search all NM conf.d directories across distros
    let dirs = [
        "/usr/lib/NetworkManager/conf.d",
        "/etc/NetworkManager/conf.d",
        "/run/NetworkManager/conf.d",
    ];

    for dir in &dirs {
        let Ok(mut entries) = tokio::fs::read_dir(dir).await else { continue };
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("conf") {
                continue;
            }
            if let Ok(content) = tokio::fs::read_to_string(&path).await {
                if let Some(uri) = parse_connectivity_uri(&content) {
                    return Ok(vec![PortalUrl {
                        url: uri,
                        source: Source::NmConnectivityConf,
                        confidence: 70,
                    }]);
                }
            }
        }
    }

    Ok(vec![])
}

fn parse_connectivity_uri(content: &str) -> Option<String> {
    let mut in_connectivity = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_connectivity = trimmed == "[connectivity]";
            continue;
        }
        if in_connectivity {
            if let Some(rest) = trimmed.strip_prefix("uri=") {
                let url = rest.trim();
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
    fn parses_connectivity_uri() {
        let content = "[connectivity]\nuri=http://connectivity-check.ubuntu.com./\ninterval=300\n";
        assert_eq!(
            parse_connectivity_uri(content),
            Some("http://connectivity-check.ubuntu.com./".to_owned())
        );
    }
}
