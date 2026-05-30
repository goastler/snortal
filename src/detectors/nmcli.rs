use crate::types::StatusNote;
use std::time::Duration;

pub async fn detect(timeout_secs: u64) -> Vec<StatusNote> {
    detect_inner(timeout_secs).await.unwrap_or_default()
}

async fn detect_inner(
    timeout_secs: u64,
) -> Result<Vec<StatusNote>, Box<dyn std::error::Error + Send + Sync>> {
    let output = tokio::time::timeout(
        Duration::from_secs(timeout_secs),
        tokio::process::Command::new("nmcli")
            .args(["-t", "-f", "CONNECTIVITY", "general", "status"])
            .output(),
    )
    .await??;

    let stdout = String::from_utf8_lossy(&output.stdout);
    // -t (terse) mode outputs just the value, one per line
    let connectivity = stdout.lines().last().unwrap_or("").trim().to_ascii_lowercase();

    Ok(match connectivity.as_str() {
        "portal" => vec![StatusNote {
            message: "nmcli reports CONNECTIVITY=portal (captive portal confirmed)".to_owned(),
        }],
        "limited" => vec![StatusNote {
            message: "nmcli reports CONNECTIVITY=limited (possible captive portal)".to_owned(),
        }],
        _ => vec![],
    })
}
