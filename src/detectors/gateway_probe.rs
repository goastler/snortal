use crate::types::{PortalUrl, Source};
use reqwest::redirect;
use std::time::Duration;

pub async fn detect(timeout_secs: u64) -> Vec<PortalUrl> {
    detect_inner(timeout_secs).await.unwrap_or_default()
}

async fn detect_inner(
    timeout_secs: u64,
) -> Result<Vec<PortalUrl>, Box<dyn std::error::Error + Send + Sync>> {
    let gateway_ip = gateway_ip_from_proc().await?;

    let client = reqwest::Client::builder()
        .redirect(redirect::Policy::none())
        .timeout(Duration::from_secs(timeout_secs))
        .build()?;

    let response = client
        .get(format!("http://{gateway_ip}/"))
        .send()
        .await?;

    if response.status().is_redirection() {
        if let Some(location) = response
            .headers()
            .get(reqwest::header::LOCATION)
            .and_then(|v| v.to_str().ok())
        {
            if url::Url::parse(location).is_ok() {
                return Ok(vec![PortalUrl {
                    url: location.to_owned(),
                    source: Source::GatewayProbe {
                        gateway_ip: gateway_ip.clone(),
                    },
                    confidence: 60,
                }]);
            }
        }
    }

    Ok(vec![])
}

async fn gateway_ip_from_proc() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let content = tokio::fs::read_to_string("/proc/net/route").await?;
    for line in content.lines().skip(1) {
        let cols: Vec<&str> = line.split_whitespace().collect();
        if cols.len() < 3 {
            continue;
        }
        // Destination column (index 1) == "00000000" means default route
        if cols[1] == "00000000" && cols[2] != "00000000" {
            return decode_hex_ip(cols[2]).ok_or("invalid gateway hex".into());
        }
    }
    Err("no default route found in /proc/net/route".into())
}

fn decode_hex_ip(hex: &str) -> Option<String> {
    // Kernel writes value as little-endian 32-bit hex on x86/x86-64.
    // Parse as u32 then to_le_bytes to get the correct octet order.
    let n = u32::from_str_radix(hex.trim(), 16).ok()?;
    let b = n.to_le_bytes();
    Some(format!("{}.{}.{}.{}", b[0], b[1], b[2], b[3]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_typical_gateway() {
        // 0101A8C0 (LE) == 192.168.1.1
        assert_eq!(decode_hex_ip("0101A8C0"), Some("192.168.1.1".to_owned()));
    }

    #[test]
    fn decodes_another_gateway() {
        // FE01A8C0 (LE) == 192.168.1.254
        assert_eq!(decode_hex_ip("FE01A8C0"), Some("192.168.1.254".to_owned()));
    }

    #[test]
    fn rejects_invalid_hex() {
        assert_eq!(decode_hex_ip("ZZZZZZZZ"), None);
    }
}
