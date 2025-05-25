mod api;
mod config;
mod settings;
mod tray;
mod utils;
mod menu;

use std::thread;
use std::time::Duration;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::Result;
use tao::event::Event;
use tao::event_loop::ControlFlow;
use tao::event_loop::EventLoopBuilder;
use tray_icon::menu::MenuEvent;

use crate::config::SharedState;
use crate::menu::UserEvent;
use crate::tray::TrayManager;

fn main() -> Result<()> {
    // 初始化共享状态
    let state = SharedState::new();

    // 创建事件循环
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let event_loop_proxy = event_loop.create_proxy();

    // 设置事件处理器
    setup_event_handlers(&event_loop);

    // 创建托盘管理器
    let mut tray_manager = TrayManager::new(state.clone());
    tray_manager.set_event_loop_proxy(event_loop_proxy.clone());

    // 初始化托盘图标（不等待数据）
    tray_manager.initialize_without_data();

    // 启动后台更新线程
    let shutdown_flag = Arc::new(AtomicBool::new(false));
    start_background_updater(&event_loop, &state, shutdown_flag.clone());

    // 运行事件循环
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(tao::event::StartCause::Init) => {
                // 发送初始数据更新事件
                let _ = event_loop_proxy.send_event(UserEvent::UpdateData);

                #[cfg(target_os = "macos")]
                {
                    use objc2_core_foundation::CFRunLoop;
                    CFRunLoop::main().unwrap();
                }
            }

            Event::UserEvent(user_event) => {
                tray_manager.handle_event(user_event, control_flow);
                
                // 检查是否需要退出
                if matches!(*control_flow, ControlFlow::Exit) {
                    shutdown_flag.store(true, Ordering::Relaxed);
                }
            }

            _ => {}
        }
    })
}

fn setup_event_handlers(event_loop: &tao::event_loop::EventLoop<UserEvent>) {
    // 设置菜单事件处理器
    let proxy = event_loop.create_proxy();
    MenuEvent::set_event_handler(Some(move |event| {
        let _ = proxy.send_event(UserEvent::MenuEvent(event));
    }));
}

fn start_background_updater(
    event_loop: &tao::event_loop::EventLoop<UserEvent>, 
    state: &SharedState,
    shutdown_flag: Arc<AtomicBool>
) {
    let state_clone = state.clone();
    let proxy = event_loop.create_proxy();

    thread::spawn(move || {
        while !shutdown_flag.load(Ordering::Relaxed) {
            // 先发送更新数据事件
            if proxy.send_event(UserEvent::UpdateData).is_err() {
                // 如果发送失败，说明事件循环已经关闭，退出线程
                break;
            }

            // 获取当前刷新间隔
            let sleep_duration = state_clone.get_refresh_interval().as_secs();
            
            // 分段睡眠，以便能够及时响应关闭信号
            let sleep_chunks = (sleep_duration / 5).max(1); // 最多分成5段
            let chunk_duration = sleep_duration / sleep_chunks;
            
            for _ in 0..sleep_chunks {
                if shutdown_flag.load(Ordering::Relaxed) {
                    return;
                }
                thread::sleep(Duration::from_secs(chunk_duration));
            }
        }
    });
}
