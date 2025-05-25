use std::fmt;
use std::sync::Arc;
use parking_lot::Mutex;

use anyhow::Result;
use chrono::Local;

use crate::api::CursorClient;
use crate::settings::Settings;

// 用于在UI中显示的用量数据
#[derive(Clone, Debug, PartialEq)]
pub struct UsageData {
    pub used: i32,
    pub total: i32,
    pub percentage: f32,
    pub email: Option<String>,
    pub last_update: String,
    pub error: Option<String>,
}

impl Default for UsageData {
    fn default() -> Self {
        Self {
            used: 0,
            total: 0,
            percentage: 0.0,
            email: None,
            last_update: Local::now().format("%H:%M:%S").to_string(),
            error: None,
        }
    }
}

impl UsageData {
    pub fn calculate_percentage(&self) -> f32 {
        if self.total == 0 {
            return 0.0;
        }
        ((self.used as f32 / self.total as f32) * 100.0 * 10.0).round() / 10.0
    }

    pub fn update_time(&mut self) {
        self.last_update = Local::now().format("%H:%M:%S").to_string();
    }
}

// 语言枚举类型
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Language {
    Chinese,
    English,
}

impl Language {
    pub fn from_str(lang: &str) -> Self {
        match lang {
            "Chinese" => Language::Chinese,
            "English" => Language::English,
            _ => Language::Chinese,
        }
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Language::Chinese => write!(f, "Chinese"),
            Language::English => write!(f, "English"),
        }
    }
}

// 刷新间隔枚举
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RefreshInterval {
    Min1 = 60,
    Min5 = 300,
    Min10 = 600,
    Min30 = 1800,
    Hour1 = 3600,
}

impl RefreshInterval {
    pub fn from_secs(secs: u64) -> Self {
        match secs {
            60 => Self::Min1,
            300 => Self::Min5,
            600 => Self::Min10,
            1800 => Self::Min30,
            3600 => Self::Hour1,
            _ => Self::Min5,
        }
    }

    pub fn as_secs(&self) -> u64 {
        *self as u64
    }

    pub fn to_string(self, lang: Language) -> &'static str {
        match lang {
            Language::Chinese => match self {
                RefreshInterval::Min1 => "1分钟",
                RefreshInterval::Min5 => "5分钟",
                RefreshInterval::Min10 => "10分钟",
                RefreshInterval::Min30 => "30分钟",
                RefreshInterval::Hour1 => "1小时",
            },
            Language::English => match self {
                RefreshInterval::Min1 => "1 minute",
                RefreshInterval::Min5 => "5 minutes",
                RefreshInterval::Min10 => "10 minutes",
                RefreshInterval::Min30 => "30 minutes",
                RefreshInterval::Hour1 => "1 hour",
            },
        }
    }

    pub fn all() -> &'static [RefreshInterval] {
        &[
            RefreshInterval::Min1,
            RefreshInterval::Min5,
            RefreshInterval::Min10,
            RefreshInterval::Min30,
            RefreshInterval::Hour1,
        ]
    }
}

// 共享状态
#[derive(Clone)]
pub struct SharedState {
    usage_data: Arc<Mutex<UsageData>>,
    language: Arc<Mutex<Language>>,
    refresh_interval: Arc<Mutex<RefreshInterval>>,
    settings: Arc<Mutex<Settings>>,
}

impl SharedState {
    pub fn new() -> Self {
        // 尝试加载设置，如果失败则使用默认设置
        let settings = match Settings::load() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to load settings: {}", e);
                Settings::default()
            }
        };

        let language = Language::from_str(&settings.language);
        let refresh_interval = RefreshInterval::from_secs(settings.refresh_interval);

        Self {
            usage_data: Arc::new(Mutex::new(UsageData::default())),
            language: Arc::new(Mutex::new(language)),
            refresh_interval: Arc::new(Mutex::new(refresh_interval)),
            settings: Arc::new(Mutex::new(settings)),
        }
    }

    pub fn get_usage_data(&self) -> UsageData {
        self.usage_data.lock().clone()
    }

    pub fn set_usage_data(&self, data: UsageData) {
        *self.usage_data.lock() = data;
    }

    pub fn get_language(&self) -> Language {
        *self.language.lock()
    }

    pub fn set_language(&self, lang: Language) {
        *self.language.lock() = lang;
        self.save_settings();
    }

    pub fn get_refresh_interval(&self) -> RefreshInterval {
        *self.refresh_interval.lock()
    }

    pub fn set_refresh_interval(&self, interval: RefreshInterval) {
        *self.refresh_interval.lock() = interval;
        self.save_settings();
    }

    pub fn update_usage_data(&self) -> Result<()> {
        let mut client = CursorClient::new();
        let data = client.fetch_usage_data()?;
        self.set_usage_data(data);
        Ok(())
    }

    fn save_settings(&self) {
        let mut settings = self.settings.lock();
        settings.language = self.get_language().to_string();
        settings.refresh_interval = self.get_refresh_interval().as_secs();

        // 尝试保存设置，失败时重试一次
        for attempt in 1..=2 {
            match settings.save() {
                Ok(_) => break,
                Err(e) => {
                    eprintln!("Failed to save settings (attempt {}): {}", attempt, e);
                    if attempt == 2 {
                        eprintln!("Settings save failed after 2 attempts, continuing without saving");
                    }
                }
            }
        }
    }
}
