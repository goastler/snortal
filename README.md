# captive-portal-finder

Detects captive portal login page URLs on a Linux network.

When connecting to a WiFi network with a captive portal, the browser redirect often fails to appear on Linux. This tool inspects the network using several strategies and prints a ranked list of URLs likely to be the portal login page.

## Usage

```
# Detect captive portal URLs on the current network
captive-portal-finder

# Show verbose output (includes SSID, detectors with no results)
captive-portal-finder --verbose

# Output as JSON
captive-portal-finder --json

# Install optional system dependencies ahead of time
captive-portal-finder install-deps
```

## Overrides

The built-in defaults for each detector can be overridden via flags. When an override is supplied it replaces the defaults entirely; omit it to use the built-in list.

### HTTP probe endpoints (`-e` / `--probe-endpoint`)

Replace the 6 built-in connectivity-check URLs with your own:

```
captive-portal-finder -e http://captive.apple.com/ -e http://connectivity-check.ubuntu.com./
```

Built-in defaults:
- `http://connectivity-check.ubuntu.com./`
- `http://captive.apple.com/`
- `http://www.gstatic.com/generate_204`
- `http://detectportal.firefox.com/success.txt`
- `http://nmcheck.gnome.org/check_network_status.txt`
- `http://network-test.debian.org/nm`

### DHCP lease files (`-d` / `--dhcp-file`)

Check specific lease files instead of the built-in paths:

```
captive-portal-finder -d /var/lib/dhcp/dhclient.leases -d /custom/path/leases
```

Built-in defaults (tried in order):
- `/var/lib/dhcp/dhclient.leases` (Debian/Ubuntu)
- `/var/lib/dhclient/dhclient.leases` (RHEL/Fedora/CentOS)
- `/var/lib/dhclient/dhclient6.leases`
- `/var/lib/NetworkManager/dhclient.conf`

### Gateway IP (`-g` / `--gateway-ip`)

Skip `/proc/net/route` auto-detection and probe a specific gateway directly:

```
captive-portal-finder --gateway-ip 192.168.0.1
```

Useful when the default route lookup fails or you know the portal is hosted on a non-default-gateway IP.

## Detection strategies

All detectors run concurrently and fail gracefully if the required file or binary is absent — the tool is fully usable even before running `install-deps`.

| Confidence | Strategy | Requires |
|---|---|---|
| 95 | DHCP option 114 — router explicitly advertised the portal URI | nothing |
| 90 | NetworkManager device file — NM parsed DHCP and stored the URI | nothing |
| 85 | HTTP redirect probe — 6 known connectivity-check endpoints redirected to portal | network |
| 70 | NM connectivity conf — reads the configured check URI | nothing |
| 60 | Gateway HTTP probe — decodes `/proc/net/route`, probes the default gateway | network |
| — | `nmcli` status — confirms `CONNECTIVITY=portal` but provides no URL | `nmcli` |
| — | WPA supplicant — SSID/state context printed in `--verbose` mode | `wpa_cli` |

HTTP probes use redirect detection (`reqwest` with `Policy::none()`): a 3xx response means the portal intercepted the request and its `Location` header is the login page URL.

## Install dependencies

The tool works without any additional packages. Two optional system binaries add extra detection context:

- **`nmcli`** — reports `CONNECTIVITY=portal` status (from NetworkManager)
- **`wpa_cli`** — shows SSID and connection state in `--verbose` mode

To install them:

```
captive-portal-finder install-deps
```

This detects your package manager (`apt`, `dnf`, `pacman`, `zypper`, `apk`) and runs the appropriate install command with `sudo`. Missing binaries are skipped.

## Building from source

```
cargo build --release
```

The binary is at `target/release/captive-portal-finder`. No runtime dependencies beyond libc — `reqwest` with `rustls-tls` is statically compiled in.

## Example output

On a network with a captive portal:

```
Captive portal URLs detected:
────────────────────────────────────────────────────────────
  95  http://192.168.1.1/portal             [dhcp-lease]
  85  http://192.168.1.1/login.html         [http-probe: connectivity-check.ubuntu.com]
  85  http://192.168.1.1/login.html         [http-probe: captive.apple.com]
  60  http://192.168.1.1/                   [gateway-probe: 192.168.1.1]
────────────────────────────────────────────────────────────
Status: nmcli reports CONNECTIVITY=portal (captive portal confirmed)
```

On a normal connected network:

```
No captive portal URLs detected.
```
