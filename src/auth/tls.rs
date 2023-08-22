use reqwest::tls::Identity;

/// Arguments for TLS client certificate authentication.
pub struct TlsOptions {
    pub cert: String,
    pub key: String,
}

impl TlsOptions {
    pub fn from_files(cert_file: String, key_file: String) -> Self {
        Self {
            cert: std::fs::read_to_string(cert_file).expect("Failed to read certificate file."),
            key: std::fs::read_to_string(key_file).expect("Failed to read key file."),
        }
    }
}

pub struct TlsState {
    cert: String,
    key: String,
}

impl TlsState {
    pub fn new(args: TlsOptions) -> Self {
        Self {
            cert: args.cert,
            key: args.key,
        }
    }

    pub fn identity(&self) -> Identity {
        let cert = self.cert.as_bytes();
        let key = self.key.as_bytes();
        Identity::from_pkcs8_pem(cert, key).expect("Failed to load TLS identity.")
    }
}
