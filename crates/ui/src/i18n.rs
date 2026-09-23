//! Internationalization (i18n) module for DB Pro native UI.

use serde::{Deserialize, Serialize};

/// Supported UI languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum UiLanguage {
    #[default]
    English,
    Vietnamese,
    Chinese,
    Japanese,
    Korean,
}

impl UiLanguage {
    pub const ALL: &'static [UiLanguage] = &[
        UiLanguage::English,
        UiLanguage::Vietnamese,
        UiLanguage::Chinese,
        UiLanguage::Japanese,
        UiLanguage::Korean,
    ];

    pub fn code(self) -> &'static str {
        match self {
            UiLanguage::English => "en",
            UiLanguage::Vietnamese => "vi",
            UiLanguage::Chinese => "zh",
            UiLanguage::Japanese => "ja",
            UiLanguage::Korean => "ko",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            UiLanguage::English => "English",
            UiLanguage::Vietnamese => "Tiếng Việt",
            UiLanguage::Chinese => "中文",
            UiLanguage::Japanese => "日本語",
            UiLanguage::Korean => "한국어",
        }
    }

    pub fn from_code(code: &str) -> Self {
        match code {
            "vi" => UiLanguage::Vietnamese,
            "zh" | "zh-CN" | "zh-Hans" => UiLanguage::Chinese,
            "ja" => UiLanguage::Japanese,
            "ko" => UiLanguage::Korean,
            _ => UiLanguage::English,
        }
    }

    /// Set global locale in `rust_i18n`.
    pub fn apply(self) {
        rust_i18n::set_locale(self.code());
    }
}
