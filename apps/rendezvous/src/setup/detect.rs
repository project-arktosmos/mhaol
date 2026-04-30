use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub enum PackageManager {
    Apt,
    Dnf,
    Yum,
}

impl PackageManager {
    pub fn name(&self) -> &'static str {
        match self {
            PackageManager::Apt => "apt",
            PackageManager::Dnf => "dnf",
            PackageManager::Yum => "yum",
        }
    }

    pub fn install_cmd(&self, package: &str) -> String {
        match self {
            PackageManager::Apt => format!("apt-get install -y {package}"),
            PackageManager::Dnf => format!("dnf install -y {package}"),
            PackageManager::Yum => format!("yum install -y {package}"),
        }
    }
}

pub struct Distro {
    pub name: String,
    pub package_manager: PackageManager,
}

pub fn detect_distro() -> Result<Distro, String> {
    let content = std::fs::read_to_string("/etc/os-release")
        .map_err(|_| "Cannot read /etc/os-release — is this a Linux system?".to_string())?;

    let fields: HashMap<String, String> = content
        .lines()
        .filter_map(|line| {
            let (key, val) = line.split_once('=')?;
            Some((key.to_string(), val.trim_matches('"').to_string()))
        })
        .collect();

    let id = fields.get("ID").cloned().unwrap_or_default();
    let id_like = fields.get("ID_LIKE").cloned().unwrap_or_default();
    let pretty_name = fields
        .get("PRETTY_NAME")
        .cloned()
        .unwrap_or_else(|| id.clone());

    let package_manager = if id == "ubuntu" || id == "debian" || id_like.contains("debian") {
        PackageManager::Apt
    } else if id == "fedora" || which_exists("dnf") {
        PackageManager::Dnf
    } else if id == "centos" || id == "rhel" || id_like.contains("rhel") || which_exists("yum") {
        PackageManager::Yum
    } else {
        return Err(format!(
            "Unsupported distribution: {pretty_name}. Supported: Ubuntu, Debian, CentOS, RHEL, Fedora"
        ));
    };

    Ok(Distro {
        name: pretty_name,
        package_manager,
    })
}

fn which_exists(cmd: &str) -> bool {
    std::process::Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
