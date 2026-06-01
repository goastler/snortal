use std::process::Command;

struct Dep {
    /// Binary name to check in PATH
    binary: &'static str,
    /// Human-readable description
    description: &'static str,
    /// Package name per package manager
    apt: &'static str,
    dnf: &'static str,
    pacman: &'static str,
    zypper: &'static str,
    apk: &'static str,
}

const DEPS: &[Dep] = &[
    Dep {
        binary: "nmcli",
        description: "NetworkManager CLI (connectivity status detection)",
        apt: "network-manager",
        dnf: "NetworkManager",
        pacman: "networkmanager",
        zypper: "NetworkManager",
        apk: "networkmanager",
    },
    Dep {
        binary: "wpa_cli",
        description: "WPA supplicant CLI (Wi-Fi connection context)",
        apt: "wpasupplicant",
        dnf: "wpa_supplicant",
        pacman: "wpa_supplicant",
        zypper: "wpa_supplicant",
        apk: "wpa_supplicant",
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PackageManager {
    Apt,
    Dnf,
    Yum,
    Pacman,
    Zypper,
    Apk,
}

impl PackageManager {
    fn detect() -> Option<Self> {
        let candidates = [
            ("apt-get", PackageManager::Apt),
            ("dnf", PackageManager::Dnf),
            ("yum", PackageManager::Yum),
            ("pacman", PackageManager::Pacman),
            ("zypper", PackageManager::Zypper),
            ("apk", PackageManager::Apk),
        ];
        for (bin, pm) in &candidates {
            if binary_in_path(bin) {
                return Some(*pm);
            }
        }
        None
    }

    fn install_cmd(&self, packages: &[&str]) -> Vec<String> {
        match self {
            PackageManager::Apt => {
                let mut cmd = vec!["sudo".to_owned(), "apt-get".to_owned(), "install".to_owned(), "-y".to_owned()];
                cmd.extend(packages.iter().map(|s| s.to_string()));
                cmd
            }
            PackageManager::Dnf => {
                let mut cmd = vec!["sudo".to_owned(), "dnf".to_owned(), "install".to_owned(), "-y".to_owned()];
                cmd.extend(packages.iter().map(|s| s.to_string()));
                cmd
            }
            PackageManager::Yum => {
                let mut cmd = vec!["sudo".to_owned(), "yum".to_owned(), "install".to_owned(), "-y".to_owned()];
                cmd.extend(packages.iter().map(|s| s.to_string()));
                cmd
            }
            PackageManager::Pacman => {
                let mut cmd = vec!["sudo".to_owned(), "pacman".to_owned(), "-S".to_owned(), "--noconfirm".to_owned()];
                cmd.extend(packages.iter().map(|s| s.to_string()));
                cmd
            }
            PackageManager::Zypper => {
                let mut cmd = vec!["sudo".to_owned(), "zypper".to_owned(), "install".to_owned(), "-y".to_owned()];
                cmd.extend(packages.iter().map(|s| s.to_string()));
                cmd
            }
            PackageManager::Apk => {
                let mut cmd = vec!["sudo".to_owned(), "apk".to_owned(), "add".to_owned()];
                cmd.extend(packages.iter().map(|s| s.to_string()));
                cmd
            }
        }
    }

    fn package_for(&self, dep: &Dep) -> &'static str {
        match self {
            PackageManager::Apt => dep.apt,
            PackageManager::Dnf | PackageManager::Yum => dep.dnf,
            PackageManager::Pacman => dep.pacman,
            PackageManager::Zypper => dep.zypper,
            PackageManager::Apk => dep.apk,
        }
    }

    fn name(&self) -> &'static str {
        match self {
            PackageManager::Apt => "apt-get",
            PackageManager::Dnf => "dnf",
            PackageManager::Yum => "yum",
            PackageManager::Pacman => "pacman",
            PackageManager::Zypper => "zypper",
            PackageManager::Apk => "apk",
        }
    }
}

pub fn run() {
    println!("Checking dependencies for portalgun...\n");

    // Check which deps are missing
    let missing: Vec<&Dep> = DEPS.iter().filter(|d| !binary_in_path(d.binary)).collect();

    if missing.is_empty() {
        println!("All dependencies are already installed:");
        for dep in DEPS {
            println!("  ✓  {} ({})", dep.binary, dep.description);
        }
        return;
    }

    // Report what's present and what's missing
    for dep in DEPS {
        if binary_in_path(dep.binary) {
            println!("  ok   {} — {}", dep.binary, dep.description);
        } else {
            println!("  miss {} — {}", dep.binary, dep.description);
        }
    }
    println!();

    let Some(pm) = PackageManager::detect() else {
        eprintln!("Could not detect a supported package manager.");
        eprintln!("Please install the following manually:");
        for dep in &missing {
            eprintln!("  {} ({})", dep.binary, dep.description);
        }
        std::process::exit(1);
    };

    let packages: Vec<&str> = missing.iter().map(|d| pm.package_for(d)).collect();
    let cmd = pm.install_cmd(&packages);

    println!("Detected package manager: {}", pm.name());
    println!("Will run: {}\n", cmd.join(" "));

    let status = Command::new(&cmd[0])
        .args(&cmd[1..])
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("\nInstallation complete.");
        }
        Ok(s) => {
            eprintln!("\nInstall command exited with status: {s}");
            std::process::exit(s.code().unwrap_or(1));
        }
        Err(e) => {
            eprintln!("\nFailed to run install command: {e}");
            std::process::exit(1);
        }
    }
}

fn binary_in_path(binary: &str) -> bool {
    Command::new("which")
        .arg(binary)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or_else(|_| path_scan(binary))
}

/// Fallback if `which` is unavailable: scan $PATH manually.
fn path_scan(binary: &str) -> bool {
    let path = std::env::var_os("PATH").unwrap_or_default();
    std::env::split_paths(&path).any(|dir| dir.join(binary).is_file())
}
