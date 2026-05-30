use std::time::Duration;

pub async fn detect(timeout_secs: u64) -> Vec<()> {
    detect_inner(timeout_secs).await.unwrap_or_default()
}

async fn detect_inner(
    timeout_secs: u64,
) -> Result<Vec<()>, Box<dyn std::error::Error + Send + Sync>> {
    // Try common wpa_supplicant socket directories
    let socket_dirs = ["/run/wpa_supplicant", "/var/run/wpa_supplicant"];

    for dir in &socket_dirs {
        let Ok(mut entries) = tokio::fs::read_dir(dir).await else { continue };
        while let Ok(Some(entry)) = entries.next_entry().await {
            let iface = entry.file_name().to_string_lossy().to_string();
            // Socket files have no extension
            if iface.contains('.') {
                continue;
            }
            let result = tokio::time::timeout(
                Duration::from_secs(timeout_secs),
                tokio::process::Command::new("wpa_cli")
                    .args(["-i", &iface, "status"])
                    .output(),
            )
            .await;

            if let Ok(Ok(out)) = result {
                let text = String::from_utf8_lossy(&out.stdout);
                let mut ssid = None;
                let mut state = None;
                for line in text.lines() {
                    if let Some(v) = line.strip_prefix("ssid=") {
                        ssid = Some(v.to_owned());
                    }
                    if let Some(v) = line.strip_prefix("wpa_state=") {
                        state = Some(v.to_owned());
                    }
                }
                if let (Some(ssid), Some(state)) = (ssid, state) {
                    eprintln!("[verbose] wpa_supplicant: iface={iface} ssid={ssid:?} state={state}");
                }
            }
        }
    }

    Ok(vec![])
}
