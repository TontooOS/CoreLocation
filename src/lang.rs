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
    fn get(&self, key: &str) -> Option<&str> {
        match key {
            "no_providers_available" => Some(&self.no_providers_available),
            "provider_failed" => Some(&self.provider_failed),
            "network_error" => Some(&self.network_error),
            "parse_error" => Some(&self.parse_error),
            "timeout" => Some(&self.timeout),
            "permission_denied" => Some(&self.permission_denied),
            _ => None,
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

pub fn t(key: &str) -> String {
    match messages().get(key) {
        Some(msg) => msg.to_string(),
        None => key.to_string(),
    }
}

pub fn t_fmt(key: &str, arg: &str) -> String {
    t(key).replace("{}", arg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_files_parse() {
        let en: Messages = serde_json::from_str(EN_US).expect("en_us.json invalid");
        let de: Messages = serde_json::from_str(DE_DE).expect("de_de.json invalid");
        assert!(!en.no_providers_available.is_empty());
        assert!(!de.no_providers_available.is_empty());
        assert_ne!(en.timeout, de.timeout);
    }

    #[test]
    fn format_replaces_placeholder() {
        assert_eq!(
            t_fmt("provider_failed", "boom"),
            t("provider_failed").replace("{}", "boom")
        );
    }

    #[test]
    fn locale_detection() {
        std::env::set_var("LANG", "de_DE.UTF-8");
        assert_eq!(current_locale(), "de_de");
        std::env::set_var("LANG", "en_US.UTF-8");
        assert_eq!(current_locale(), "en_us");
        std::env::remove_var("LANG");
    }
}
