mod coturn;
mod detect;
mod systemd;
mod tls;

use dialoguer::{Confirm, Input};

pub fn run_wizard() -> Result<(), String> {
    println!("\n=== Mhaol Signaling Server Setup ===\n");

    // Step 1: Detect OS
    let distro = detect::detect_distro()?;
    println!("Detected: {} ({})", distro.name, distro.package_manager.name());

    // Step 2: Domain name
    let domain: String = Input::new()
        .with_prompt("Domain name for this server (e.g., signal.example.com)")
        .interact_text()
        .map_err(|e| format!("Input error: {e}"))?;

    // Step 3: Install coturn
    if Confirm::new()
        .with_prompt("Install coturn TURN server?")
        .default(true)
        .interact()
        .map_err(|e| format!("Input error: {e}"))?
    {
        coturn::install(&distro)?;
    }

    // Step 4: Generate shared secret
    let shared_secret = generate_secret();
    println!("Generated TURN shared secret: {shared_secret}");

    // Step 5: Generate API key
    let api_key = generate_secret();
    println!("Generated API key: {api_key}");

    // Step 6: TLS setup
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

    // Step 7: Write coturn config
    coturn::write_config(&domain, &shared_secret, cert_path.as_deref(), key_path.as_deref())?;

    // Step 8: Write signaling config
    let port: u16 = Input::new()
        .with_prompt("Signaling server port")
        .default(8443)
        .interact_text()
        .map_err(|e| format!("Input error: {e}"))?;

    let config_content = format!(
        r#"[server]
host = "0.0.0.0"
port = {port}
{}
[turn]
domain = "{domain}"
shared_secret = "{shared_secret}"
credential_ttl_secs = 86400
stun_port = 3478
turn_port = 3478
turns_port = 5349

[auth]
api_keys = ["{api_key}"]
"#,
        match (&cert_path, &key_path) {
            (Some(c), Some(k)) => format!("tls_cert = \"{c}\"\ntls_key = \"{k}\"\n"),
            _ => String::new(),
        }
    );

    let config_dir = "/etc/mhaol-signaling";
    let config_path = format!("{config_dir}/config.toml");
    std::fs::create_dir_all(config_dir)
        .map_err(|e| format!("Failed to create {config_dir}: {e}"))?;
    std::fs::write(&config_path, &config_content)
        .map_err(|e| format!("Failed to write config: {e}"))?;
    println!("Wrote signaling config to {config_path}");

    // Step 9: Systemd services
    if Confirm::new()
        .with_prompt("Create and enable systemd services?")
        .default(true)
        .interact()
        .map_err(|e| format!("Input error: {e}"))?
    {
        systemd::install_service(&config_path)?;
        coturn::enable_service()?;
    }

    // Summary
    let protocol = if cert_path.is_some() { "https" } else { "http" };
    println!("\n=== Setup Complete ===\n");
    println!("Signaling URL:  {protocol}://{domain}:{port}");
    println!("TURN domain:    {domain}");
    println!("API key:        {api_key}");
    println!("Config file:    {config_path}");
    println!("\nTo use with your mhaol server, set these environment variables:");
    println!("  SIGNALING_URL={protocol}://{domain}:{port}");
    println!("  TURN_CREDENTIAL_URL={protocol}://{domain}:{port}/api/v1/turn/credentials?apiKey={api_key}");

    Ok(())
}

fn generate_secret() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
