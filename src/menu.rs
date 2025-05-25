use std::collections::HashMap;
use tray_icon::menu::{Menu, MenuItem, MenuEvent};
use tao::event_loop::ControlFlow;
use tao::event_loop::EventLoopProxy;

use crate::config::{Language, RefreshInterval, UsageData, SharedState};

// 用户事件枚举
pub enum UserEvent {
    MenuEvent(MenuEvent),
    UpdateData,
    UpdateTrayIcon,
}

// 菜单动作枚举
#[derive(Debug, Clone)]
pub enum MenuAction {
    Refresh,
    OpenSettings,
    Quit,
    SetLanguage(Language),
    SetInterval(RefreshInterval),
}

impl MenuAction {
    pub fn from_string(s: &str) -> Option<Self> {
        match s {
            "refresh" => Some(MenuAction::Refresh),
            "open_settings" => Some(MenuAction::OpenSettings),
            "quit" => Some(MenuAction::Quit),
            "lang_chinese" => Some(MenuAction::SetLanguage(Language::Chinese)),
            "lang_english" => Some(MenuAction::SetLanguage(Language::English)),
            s if s.starts_with("interval_") => match s.strip_prefix("interval_")? {
                "Min1" => Some(MenuAction::SetInterval(RefreshInterval::Min1)),
                "Min5" => Some(MenuAction::SetInterval(RefreshInterval::Min5)),
                "Min10" => Some(MenuAction::SetInterval(RefreshInterval::Min10)),
                "Min30" => Some(MenuAction::SetInterval(RefreshInterval::Min30)),
                "Hour1" => Some(MenuAction::SetInterval(RefreshInterval::Hour1)),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn handle(&self, state: &SharedState, event_loop_proxy: &Option<EventLoopProxy<UserEvent>>, control_flow: &mut ControlFlow) {
        match self {
            MenuAction::Refresh => {
                if let Some(proxy) = event_loop_proxy {
                    let _ = proxy.send_event(UserEvent::UpdateData);
                }
            }
            MenuAction::OpenSettings => {
                let _ = open::that("https://www.cursor.com/settings");
            }
            MenuAction::Quit => {
                *control_flow = ControlFlow::Exit;
            }
            MenuAction::SetLanguage(lang) => {
                state.set_language(*lang);
                if let Some(proxy) = event_loop_proxy {
                    let _ = proxy.send_event(UserEvent::UpdateTrayIcon);
                }
            }
            MenuAction::SetInterval(interval) => {
                state.set_refresh_interval(*interval);
                if let Some(proxy) = event_loop_proxy {
                    let _ = proxy.send_event(UserEvent::UpdateTrayIcon);
                }
            }
        }
    }
}

// 菜单文本
#[derive(Clone, Copy)]
pub struct MenuTexts {
    pub title: &'static str,
    pub refresh: &'static str,
    pub settings: &'static str,
    pub quit: &'static str,
    pub language: &'static str,
    pub refresh_interval: &'static str,
    pub used: &'static str,
    pub remaining: &'static str,
    pub usage_rate: &'static str,
    pub account: &'static str,
    pub last_update: &'static str,
    pub requests: &'static str,
    pub options: &'static str,
}

impl Language {
    pub fn get_menu_texts(&self) -> MenuTexts {
        match self {
            Language::Chinese => MenuTexts {
                title: "🤖 Cursor GPT-4 用量",
                language: "----- 🇺🇳 语言 -----",
                refresh_interval: "----- ⏳ 刷新间隔 -----",
                used: "已用",
                remaining: "剩余",
                usage_rate: "使用率",
                account: "账户",
                last_update: "最后更新",
                requests: "次请求",
                options: "----- ⚙️ 选项 -----",
                refresh: "刷新数据",
                settings: "打开Cursor设置",
                quit: "退出",
            },
            Language::English => MenuTexts {
                title: "🤖 Cursor GPT-4 Usage",
                language: "-----🇺🇳 Language -----",
                refresh_interval: "----- ⏳ Refresh Interval -----",
                used: "Used",
                remaining: "Remaining",
                usage_rate: "Usage",
                account: "Account",
                last_update: "Last updated",
                requests: "requests",
                options: "----- ⚙️ Options -----",
                refresh: "Refresh Data",
                settings: "Open Cursor Settings",
                quit: "Exit",
            },
        }
    }
}

// 菜单构建器
pub struct MenuBuilder {
    menu: Menu,
    actions: HashMap<String, String>,
    language: Language,
    refresh_interval: RefreshInterval,
    usage_data: UsageData,
}

impl MenuBuilder {
    pub fn new(language: Language, refresh_interval: RefreshInterval, usage_data: UsageData) -> Self {
        Self {
            menu: Menu::new(),
            actions: HashMap::new(),
            language,
            refresh_interval,
            usage_data,
        }
    }

    pub fn build(mut self) -> (Menu, HashMap<String, String>) {
        self.add_title()
            .add_usage_info()
            .add_refresh_interval_menu()
            .add_language_menu()
            .add_options_menu();

        (self.menu, self.actions)
    }

    fn add_title(&mut self) -> &mut Self {
        let texts = self.language.get_menu_texts();
        self.menu.append(&MenuItem::new(texts.title, false, None)).unwrap();
        self
    }

    fn add_usage_info(&mut self) -> &mut Self {
        let texts = self.language.get_menu_texts();

        if let Some(error) = &self.usage_data.error {
            self.menu.append(&MenuItem::new(format!("❌ Error: {}", error), false, None)).unwrap();
        } else {
            let used_text = format!("{}: {}/{} {}", texts.used, self.usage_data.used, self.usage_data.total, texts.requests);
            let remaining_text = format!("{}: {} {}", texts.remaining, self.usage_data.total - self.usage_data.used, texts.requests);
            let percentage_text = format!("{}: {:.1}%", texts.usage_rate, self.usage_data.percentage);

            self.menu.append(&MenuItem::new(used_text, false, None)).unwrap();
            self.menu.append(&MenuItem::new(remaining_text, false, None)).unwrap();
            self.menu.append(&MenuItem::new(percentage_text, false, None)).unwrap();

            if let Some(email) = &self.usage_data.email {
                let email_text = format!("{}: {}", texts.account, email);
                self.menu.append(&MenuItem::new(email_text, false, None)).unwrap();
            }

            let update_time_text = format!("{}: {}", texts.last_update, self.usage_data.last_update);
            self.menu.append(&MenuItem::new(update_time_text, false, None)).unwrap();
        }
        self
    }

    fn add_refresh_interval_menu(&mut self) -> &mut Self {
        let texts = self.language.get_menu_texts();
        self.menu.append(&MenuItem::new(texts.refresh_interval, false, None)).unwrap();

        for &interval in RefreshInterval::all() {
            let check_mark = if interval == self.refresh_interval { "✓ " } else { "    " };
            let text = format!("{}{}", check_mark, interval.to_string(self.language));
            let item = MenuItem::new(text, true, None);
            let id = item.id().0.to_string();
            self.actions.insert(id.clone(), format!("interval_{:?}", interval));
            self.menu.append(&item).unwrap();
        }
        self
    }

    fn add_language_menu(&mut self) -> &mut Self {
        let texts = self.language.get_menu_texts();
        self.menu.append(&MenuItem::new(texts.language, false, None)).unwrap();

        // 中文选项
        let chinese_item = MenuItem::new(
            format!("{}中文", if self.language == Language::Chinese { "✓ " } else { "    " }),
            true,
            None,
        );
        let chinese_id = chinese_item.id().0.to_string();
        self.actions.insert(chinese_id, "lang_chinese".to_string());
        self.menu.append(&chinese_item).unwrap();

        // 英文选项
        let english_item = MenuItem::new(
            format!("{}English", if self.language == Language::English { "✓ " } else { "    " }),
            true,
            None,
        );
        let english_id = english_item.id().0.to_string();
        self.actions.insert(english_id, "lang_english".to_string());
        self.menu.append(&english_item).unwrap();
        self
    }

    fn add_options_menu(&mut self) -> &mut Self {
        let texts = self.language.get_menu_texts();
        self.menu.append(&MenuItem::new(texts.options, false, None)).unwrap();

        let refresh_item = MenuItem::new(texts.refresh, true, None);
        let refresh_id = refresh_item.id().0.to_string();
        self.actions.insert(refresh_id, "refresh".to_string());
        self.menu.append(&refresh_item).unwrap();

        let settings_item = MenuItem::new(texts.settings, true, None);
        let settings_id = settings_item.id().0.to_string();
        self.actions.insert(settings_id, "open_settings".to_string());
        self.menu.append(&settings_item).unwrap();

        let quit_item = MenuItem::new(texts.quit, true, None);
        let quit_id = quit_item.id().0.to_string();
        self.actions.insert(quit_id, "quit".to_string());
        self.menu.append(&quit_item).unwrap();
        self
    }
} 