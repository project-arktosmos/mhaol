use super::detect::Distro;

pub fn setup_certbot(distro: &Distro, domain: &str) -> Result<(Option<String>, Option<String>), String> {
    println!("Installing certbot...");

    let install_cmd = match distro.package_manager {
        super::detect::PackageManager::Apt => "apt-get install -y certbot",
        super::detect::PackageManager::Dnf => "dnf install -y certbot",
        super::detect::PackageManager::Yum => "yum install -y certbot",
    };
    run_shell(install_cmd)?;

    println!("Requesting certificate for {domain}...");
    println!("Note: Port 80 must be available for HTTP challenge.");
    let certbot_cmd = format!("certbot certonly --standalone -d {domain} --non-interactive --agree-tos --register-unsafely-without-email");
    run_shell(&certbot_cmd)?;

    let cert = format!("/etc/letsencrypt/live/{domain}/fullchain.pem");
    let key = format!("/etc/letsencrypt/live/{domain}/privkey.pem");

    if std::path::Path::new(&cert).exists() && std::path::Path::new(&key).exists() {
        println!("TLS certificates obtained");
        Ok((Some(cert), Some(key)))
    } else {
        Err("certbot ran but certificates not found at expected paths".into())
    }
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
