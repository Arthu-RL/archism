use std::sync::{Arc, Mutex};
use tokio::process::Command;

use state::{AppState, Step};

pub async fn discover_wifi(state: Arc<Mutex<AppState>>) {
    let _ = Command::new("iwctl").args(["station", "wlan0", "scan"]).output().await;
    let output = Command::new("iwctl")
        .args(["station", "wlan0", "get-networks"])
        .output()
        .await;

    if let Ok(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout);
        let mut networks = vec!["Usa conexão com fio (Ethernet) / Pular".to_string()];

        for line in stdout.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty()
                || trimmed.contains("Available networks")
                || trimmed.contains("---")
                || trimmed.contains("Network name")
            {
                continue;
            }

            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if !parts.is_empty() {
                let ssid = parts[0].to_string();
                if !networks.contains(&ssid) {
                    networks.push(ssid);
                }
            }
        }

        let mut s = state.lock().unwrap();
        s.wifi_list = networks;
    }
}

pub async fn connect_wifi(state: Arc<Mutex<AppState>>, ssid: String, pass: String) {
    {
        let mut s = state.lock().unwrap();
        s.wifi_status = format!("Conectando a {}...", ssid);
    }

    let status = if pass.is_empty() {
        Command::new("iwctl")
            .args(["station", "wlan0", "connect", &ssid])
            .output()
            .await
    } else {
        Command::new("iwctl")
            .args(["--passphrase", &pass, "station", "wlan0", "connect", &ssid])
            .output()
            .await
    };

    let mut s = state.lock().unwrap();
    match status {
        Ok(out) if out.status.success() => {
            s.wifi_status = format!("✓ Conectado com sucesso a {}!", ssid);
            s.wifi_input_mode = false;
            s.step = Step::Configure;
        }
        _ => {
            s.wifi_status = format!("✕ Falha ao conectar em {}. Verifique a senha.", ssid);
        }
    }
}