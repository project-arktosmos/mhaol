const SERVICE_NAME: &str = "mhaol-signaling";

pub fn install_service(config_path: &str) -> Result<(), String> {
    let exe_path = std::env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "/usr/local/bin/mhaol-signaling".into());

    let unit = format!(
        r#"[Unit]
Description=Mhaol Signaling Server
After=network.target

[Service]
Type=simple
ExecStart={exe_path} serve --config {config_path}
Restart=always
RestartSec=5
LimitNOFILE=65535

[Install]
WantedBy=multi-user.target
"#
    );

    let service_path = format!("/etc/systemd/system/{SERVICE_NAME}.service");
    std::fs::write(&service_path, &unit)
        .map_err(|e| format!("Failed to write {service_path}: {e}"))?;
    println!("Wrote systemd unit to {service_path}");

    run_shell("systemctl daemon-reload")?;
    run_shell(&format!("systemctl enable {SERVICE_NAME}"))?;
    run_shell(&format!("systemctl restart {SERVICE_NAME}"))?;
    println!("{SERVICE_NAME} service enabled and started");

    Ok(())
}

fn run_shell(cmd: &str) -> Result<(), String> {
    let status = std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .status()
        .map_err(|e| format!("Failed to run '{cmd}': {e}"))?;
    if !status.success() {
        return Err(format!("Command failed: {cmd}"));
    }
    Ok(())
}
