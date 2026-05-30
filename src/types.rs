#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    DhcpLease,
    NetworkManagerDevice,
    HttpProbe { endpoint: String },
    NmConnectivityConf,
    GatewayProbe { gateway_ip: String },
}

impl std::fmt::Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Source::DhcpLease => write!(f, "dhcp-lease"),
            Source::NetworkManagerDevice => write!(f, "networkmanager-device"),
            Source::HttpProbe { endpoint } => write!(f, "http-probe: {endpoint}"),
            Source::NmConnectivityConf => write!(f, "nm-connectivity-conf"),
            Source::GatewayProbe { gateway_ip } => write!(f, "gateway-probe: {gateway_ip}"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PortalUrl {
    pub url: String,
    pub source: Source,
    pub confidence: u8,
}

/// Returned by detectors that find network status without a URL.
#[derive(Debug, Clone)]
pub struct StatusNote {
    pub message: String,
}
