mod detectors;
mod install;
mod output;
mod types;

#[derive(clap::Parser, Debug)]
#[command(
    name = "captive-portal-finder",
    about = "Detect captive portal URLs on the current Linux network",
    version
)]
struct Cli {
    /// Timeout in seconds for HTTP requests and subprocess calls
    #[arg(short, long, default_value = "5")]
    timeout: u64,

    /// Show verbose output including SSID and detectors with no results
    #[arg(short, long)]
    verbose: bool,

    /// Output results as JSON
    #[arg(long)]
    json: bool,

    /// HTTP probe endpoints to use instead of the built-in defaults
    /// (e.g. http://captive.apple.com/ http://connectivity-check.ubuntu.com./)
    #[arg(short = 'e', long = "probe-endpoint", value_name = "URL")]
    probe_endpoints: Vec<String>,

    /// DHCP lease files to check instead of the built-in defaults
    /// (e.g. /var/lib/dhcp/dhclient.leases)
    #[arg(short = 'd', long = "dhcp-file", value_name = "PATH")]
    dhcp_files: Vec<String>,

    /// Gateway IP to probe directly, skipping /proc/net/route auto-detection
    /// (e.g. 192.168.1.1)
    #[arg(short = 'g', long = "gateway-ip", value_name = "IP")]
    gateway_ip: Option<String>,

    #[command(subcommand)]
    command: Option<Subcommand>,
}

#[derive(clap::Subcommand, Debug)]
enum Subcommand {
    /// Install optional system dependencies (nmcli, wpa_cli) using the system package manager
    InstallDeps,
}

#[tokio::main]
async fn main() {
    let cli = <Cli as clap::Parser>::parse();

    if let Some(Subcommand::InstallDeps) = cli.command {
        install::run();
        return;
    }

    let results = detectors::run_all(&detectors::DetectorConfig {
        timeout_secs: cli.timeout,
        verbose: cli.verbose,
        probe_endpoints: cli.probe_endpoints,
        dhcp_files: cli.dhcp_files,
        gateway_ip: cli.gateway_ip,
    })
    .await;
    output::print_results(results.urls, results.notes, cli.json, cli.verbose);
}
