use image::ImageBuffer;
use image::Rgba;
use tao::event_loop::ControlFlow;
use tao::event_loop::EventLoopProxy;
use tray_icon::Icon;
use tray_icon::TrayIcon;
use tray_icon::TrayIconBuilder;

use crate::config::SharedState;
use crate::config::UsageData;
use crate::menu::{MenuBuilder, UserEvent, MenuAction};

use std::thread;

pub struct TrayManager {
    tray_icon: Option<TrayIcon>,
    state: SharedState,
    event_loop_proxy: Option<EventLoopProxy<UserEvent>>,
    menu_actions: std::collections::HashMap<String, String>,
}

impl TrayManager {
    pub fn new(state: SharedState) -> Self {
        Self { 
            tray_icon: None, 
            state, 
            event_loop_proxy: None,
            menu_actions: std::collections::HashMap::new(),
        }
    }

    pub fn set_event_loop_proxy(&mut self, proxy: EventLoopProxy<UserEvent>) {
        self.event_loop_proxy = Some(proxy);
    }

    pub fn initialize_without_data(&mut self) {
        // 创建一个默认的灰色图标
        let icon = create_default_icon();
        let lang = self.state.get_language();
        let interval = self.state.get_refresh_interval();
        let data = UsageData::default();
        
        let (menu, actions) = MenuBuilder::new(lang, interval, data).build();
        self.menu_actions = actions;
        
        self.tray_icon = Some(TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_icon(icon)
            .build()
            .unwrap());
    }

    pub fn handle_event(&mut self, event: UserEvent, control_flow: &mut ControlFlow) {
        match event {
            UserEvent::MenuEvent(event) => {
                if let Some(action_str) = self.menu_actions.get(&event.id.0) {
                    if let Some(action) = MenuAction::from_string(action_str) {
                        action.handle(&self.state, &self.event_loop_proxy, control_flow);
                    }
                }
            }
            UserEvent::UpdateData => {
                // 异步更新数据，不阻塞UI
                let state = self.state.clone();
                let proxy = self.event_loop_proxy.clone();
                
                thread::spawn(move || {
                    if let Err(e) = state.update_usage_data() {
                        eprintln!("Failed to update data: {}", e);
                    }
                    if let Some(proxy) = proxy {
                        let _ = proxy.send_event(UserEvent::UpdateTrayIcon);
                    }
                });
            }
            UserEvent::UpdateTrayIcon => {
                self.update_tray_icon();
            }
        }
    }

    fn update_tray_icon(&mut self) {
        let data = self.state.get_usage_data();
        let lang = self.state.get_language();
        let interval = self.state.get_refresh_interval();

        let icon = create_icon(&data);
        let (menu, actions) = MenuBuilder::new(lang, interval, data).build();

        self.menu_actions = actions;
        if let Some(tray_icon) = &mut self.tray_icon {
            if let Err(e) = tray_icon.set_icon(Some(icon)) {
                eprintln!("Failed to update tray icon: {}", e);
            }
            tray_icon.set_menu(Some(Box::new(menu)));
        }
    }
}

fn create_default_icon() -> Icon {
    let icon_size = 32;
    let icon = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_fn(icon_size, icon_size, |x, y| {
        let center_x = icon_size as f32 / 2.0;
        let center_y = icon_size as f32 / 2.0;
        let distance = ((x as f32 - center_x).powi(2) + (y as f32 - center_y).powi(2)).sqrt();

        if distance < (icon_size as f32 / 2.5) {
            Rgba([128, 128, 128, 255]) // 灰色
        } else {
            Rgba([0, 0, 0, 0])
        }
    });

    Icon::from_rgba(icon.into_raw(), icon_size, icon_size)
        .unwrap_or_else(|e| {
            eprintln!("Failed to create default icon: {}, using fallback", e);
            // 创建一个最简单的1x1像素图标作为fallback
            Icon::from_rgba(vec![128, 128, 128, 255], 1, 1)
                .expect("Failed to create fallback icon")
        })
}

fn create_icon(usage_data: &UsageData) -> Icon {
    let percentage = usage_data.percentage;
    let (r, g, b) = get_color_for_usage(percentage);

    let icon_size = 32;
    let icon = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_fn(icon_size, icon_size, |x, y| {
        let center_x = icon_size as f32 / 2.0;
        let center_y = icon_size as f32 / 2.0;
        let distance = ((x as f32 - center_x).powi(2) + (y as f32 - center_y).powi(2)).sqrt();

        if distance < (icon_size as f32 / 2.5) { 
            Rgba([r, g, b, 255]) 
        } else { 
            Rgba([0, 0, 0, 0]) 
        }
    });

    Icon::from_rgba(icon.into_raw(), icon_size, icon_size)
        .unwrap_or_else(|e| {
            eprintln!("Failed to create usage icon: {}, using default", e);
            create_default_icon()
        })
}

fn get_color_for_usage(percentage: f32) -> (u8, u8, u8) {
    if percentage >= 90.0 {
        (230, 40, 40) // 红色
    } else if percentage >= 70.0 {
        (250, 150, 30) // 橙色
    } else if percentage >= 50.0 {
        (250, 230, 30) // 黄色
    } else {
        (50, 200, 100) // 绿色
    }
}
