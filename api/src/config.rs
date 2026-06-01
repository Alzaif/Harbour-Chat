use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Config {
    pub port: u16,
    pub db_path: PathBuf,
    pub data_dir: PathBuf,
    pub trust_gateway_headers: bool,
    pub dev_user_id: Option<String>,
    pub dev_user_email: Option<String>,
    pub dev_user_display_name: Option<String>,
    pub package_name: String,
    pub package_version: String,
    pub max_attachment_bytes: u64,
    pub require_https_forwarded_proto: bool,
    pub trusted_proxy_token: Option<String>,
    pub master_key_b64: Option<String>,
    pub master_key_id: String,
    pub enable_security_audit_log: bool,
    pub quarantine_suspicious_attachments: bool,
    pub voice_sfu_base_url: String,
    pub voice_turn_urls: Vec<String>,
    pub voice_turn_secret: String,
    pub voice_turn_ttl_seconds: i64,
    /// Internal Portcullis ForwardAuth URL used for WebSocket session validation.
    pub portcullis_forward_url: String,
    /// Host header sent to Portcullis when resolving app scope for WebSocket auth.
    pub forward_auth_host: String,
}

impl Config {
    pub fn from_env() -> Self {
        let trust_gateway_headers = std::env::var("TRUST_GATEWAY_HEADERS")
            .map(|v| v == "true")
            .unwrap_or(true);

        Self {
            port: std::env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3004),
            db_path: PathBuf::from(
                std::env::var("CHAT_DB_PATH").unwrap_or_else(|_| "./data/chat.db".into()),
            ),
            data_dir: PathBuf::from(
                std::env::var("CHAT_DATA_DIR").unwrap_or_else(|_| "./data".into()),
            ),
            trust_gateway_headers,
            dev_user_id: std::env::var("DEV_USER_ID").ok(),
            dev_user_email: std::env::var("DEV_USER_EMAIL").ok(),
            dev_user_display_name: std::env::var("DEV_USER_DISPLAY_NAME").ok(),
            package_name: "harbour-chat".into(),
            package_version: env!("CARGO_PKG_VERSION").into(),
            max_attachment_bytes: std::env::var("CHAT_MAX_ATTACHMENT_BYTES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10 * 1024 * 1024),
            require_https_forwarded_proto: std::env::var("CHAT_REQUIRE_HTTPS_FORWARDED_PROTO")
                .map(|v| v != "false" && v != "0")
                .unwrap_or(true),
            trusted_proxy_token: std::env::var("CHAT_TRUSTED_PROXY_TOKEN").ok(),
            master_key_b64: std::env::var("CHAT_MASTER_KEY_B64").ok(),
            master_key_id: std::env::var("CHAT_MASTER_KEY_ID").unwrap_or_else(|_| "local-v1".into()),
            enable_security_audit_log: std::env::var("CHAT_ENABLE_SECURITY_AUDIT_LOG")
                .map(|v| v != "false" && v != "0")
                .unwrap_or(true),
            quarantine_suspicious_attachments: std::env::var(
                "CHAT_QUARANTINE_SUSPICIOUS_ATTACHMENTS",
            )
            .map(|v| v != "false" && v != "0")
            .unwrap_or(true),
            voice_sfu_base_url: std::env::var("CHAT_VOICE_SFU_BASE_URL")
                .unwrap_or_else(|_| "http://harbour-chat-sfu:4000".into()),
            voice_turn_urls: std::env::var("CHAT_VOICE_TURN_URLS")
                .map(|v| {
                    v.split(',')
                        .map(|u| u.trim().to_string())
                        .filter(|u| !u.is_empty())
                        .collect()
                })
                .unwrap_or_else(|_| vec![]),
            voice_turn_secret: std::env::var("CHAT_VOICE_TURN_SECRET")
                .unwrap_or_else(|_| "harbour-dev-turn-secret".into()),
            voice_turn_ttl_seconds: std::env::var("CHAT_VOICE_TURN_TTL_SECONDS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3600),
            portcullis_forward_url: std::env::var("CHAT_PORTCULLIS_FORWARD_URL")
                .unwrap_or_else(|_| "http://harbour-portcullis:3000/auth/forward".into()),
            forward_auth_host: std::env::var("CHAT_FORWARD_AUTH_HOST")
                .unwrap_or_else(|_| "chat.harbour.local".into()),
        }
    }

    pub fn for_test(db_path: PathBuf) -> Self {
        Self {
            port: 0,
            db_path,
            data_dir: std::env::temp_dir().join("harbour-chat-test"),
            trust_gateway_headers: false,
            dev_user_id: Some("dev-user".into()),
            dev_user_email: Some("dev@harbour.local".into()),
            dev_user_display_name: Some("Dev User".into()),
            package_name: "harbour-chat".into(),
            package_version: "test".into(),
            max_attachment_bytes: 10 * 1024 * 1024,
            require_https_forwarded_proto: false,
            trusted_proxy_token: None,
            master_key_b64: Some("QkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkI=".into()),
            master_key_id: "test-v1".into(),
            enable_security_audit_log: true,
            quarantine_suspicious_attachments: true,
            voice_sfu_base_url: "http://localhost:4000".into(),
            voice_turn_urls: vec!["stun:stun.l.google.com:19302".into()],
            voice_turn_secret: "harbour-dev-turn-secret".into(),
            voice_turn_ttl_seconds: 3600,
            portcullis_forward_url: "http://127.0.0.1:1/auth/forward".into(),
            forward_auth_host: "chat.harbour.local".into(),
        }
    }
}
