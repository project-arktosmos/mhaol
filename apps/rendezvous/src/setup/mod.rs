mod coturn;
mod detect;
mod systemd;
mod tls;

use dialoguer::{Confirm, Input};

/// Interactive Linux deployment wizard. Installs coturn, optionally provisions
/// Let's Encrypt certificates, writes a TOML config, and registers a systemd
/// service called `mhaol-rendezvous`.
pub fn run_wizard() -> Result<(), String> {
    println!("\n=== Mhaol Rendezvous Setup ===\n");

    let distro = detect::detect_distro()?;
    println!("Detected: {} ({})", distro.name, distro.package_manager.name());

    let domain: String = Input::new()
        .with_prompt("Domain name for this server (e.g., rendezvous.example.com)")
        .interact_text()
        .map_err(|e| format!("Input error: {e}"))?;

    if Confirm::new()
        .with_prompt("Install coturn TURN server?")
        .default(true)
        .interact()
        .map_err(|e| format!("Input error: {e}"))?
    {
        coturn::install(&distro)?;
    }

    let shared_secret = generate_secret();
    println!("Generated TURN shared secret: {shared_secret}");

    let api_key = generate_secret();
    println!("Generated API key: {api_key}");

    let (cert_path, key_path) = if Confirm::new()
        .with_prompt("Set up TLS with Let's Encrypt (certbot)?")
        .default(true)
        .interact()
        .map_err(|e| format!("Input error: {e}"))?
    {
        tls::setup_certbot(&distro, &domain)?
    } else {
        let cert: String = Input::new()
            .with_prompt("TLS certificate path (or leave empty to skip TLS)")
            .allow_empty(true)
            .interact_text()
            .map_err(|e| format!("Input error: {e}"))?;
        if cert.is_empty() {
            (None, None)
        } else {
            let key: String = Input::new()
                .with_prompt("TLS private key path")
                .interact_text()
                .map_err(|e| format!("Input error: {e}"))?;
            (Some(cert), Some(key))
        }
    };

    coturn::write_config(&domain, &shared_secret, cert_path.as_deref(), key_path.as_deref())?;

    let port: u16 = Input::new()
        .with_prompt("Rendezvous HTTP port")
        .default(14080)
        .interact_text()
        .map_err(|e| format!("Input error: {e}"))?;

    let config_content = format!(
        r#"[server]
host = "0.0.0.0"
http_port = {port}
ipfs_listen_port = 14001
{}
[turn]
domain = "{domain}"
shared_secret = "{shared_secret}"
credential_ttl_secs = 86400
stun_port = 3478
turn_port = 3478
turns_port = 5349
api_keys = ["{api_key}"]
"#,
        match (&cert_path, &key_path) {
            (Some(c), Some(k)) => format!("tls_cert = \"{c}\"\ntls_key = \"{k}\"\n"),
            _ => String::new(),
        }
    );

    let config_dir = "/etc/mhaol-rendezvous";
    let config_path = format!("{config_dir}/config.toml");
    std::fs::create_dir_all(config_dir)
        .map_err(|e| format!("Failed to create {config_dir}: {e}"))?;
    std::fs::write(&config_path, &config_content)
        .map_err(|e| format!("Failed to write config: {e}"))?;
    println!("Wrote rendezvous config to {config_path}");

    if Confirm::new()
        .with_prompt("Create and enable systemd services?")
        .default(true)
        .interact()
        .map_err(|e| format!("Input error: {e}"))?
    {
        systemd::install_service(&config_path)?;
        coturn::enable_service()?;
    }

    let protocol = if cert_path.is_some() { "https" } else { "http" };
    println!("\n=== Setup Complete ===\n");
    println!("Rendezvous URL: {protocol}://{domain}:{port}");
    println!("TURN domain:    {domain}");
    println!("API key:        {api_key}");
    println!("Config file:    {config_path}");
    println!("\nTo use with your mhaol cloud, set these environment variables:");
    println!("  SIGNALING_URL={protocol}://{domain}:{port}");
    println!(
        "  TURN_CREDENTIAL_URL={protocol}://{domain}:{port}/api/v1/turn/credentials?apiKey={api_key}"
    );

    Ok(())
}

fn generate_secret() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
