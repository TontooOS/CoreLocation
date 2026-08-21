use serde::Deserialize;
use std::sync::OnceLock;

const EN_US: &str = include_str!("../lang/en_us.json");
const DE_DE: &str = include_str!("../lang/de_de.json");

#[derive(Deserialize)]
struct Messages {
    no_providers_available: String,
    provider_failed: String,
    network_error: String,
    parse_error: String,
    timeout: String,
    permission_denied: String,
}

impl Messages {
    fn get(&self, key: &str) -> &str {
        match key {
            "no_providers_available" => &self.no_providers_available,
            "provider_failed" => &self.provider_failed,
            "network_error" => &self.network_error,
            "parse_error" => &self.parse_error,
            "timeout" => &self.timeout,
            "permission_denied" => &self.permission_denied,
            _ => key,
        }
    }
}

static MESSAGES: OnceLock<Messages> = OnceLock::new();

pub fn current_locale() -> &'static str {
    let lang = std::env::var("LC_ALL")
        .or_else(|_| std::env::var("LC_MESSAGES"))
        .or_else(|_| std::env::var("LANG"))
        .unwrap_or_default();

    if lang.to_lowercase().starts_with("de") {
        "de_de"
    } else {
        "en_us"
    }
}

fn messages() -> &'static Messages {
    MESSAGES.get_or_init(|| {
        let raw = match current_locale() {
            "de_de" => DE_DE,
            _ => EN_US,
        };
        serde_json::from_str(raw).expect("built-in language file is invalid")
    })
}

pub fn t(key: &str) -> &'static str {
    messages().get(key)
}

pub fn t_fmt(key: &str, arg: &str) -> String {
    t(key).replace("{}", arg)
}
