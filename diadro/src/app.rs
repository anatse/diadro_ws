use std::time::Duration;

use crate::graph::Graphics;
use eframe::egui::{Id, LayerId, Ui, Vec2};
use eframe::{egui, epi};

#[cfg(target_arch = "wasm32")]
use wasm_sockets::{self, WebSocketError};

pub struct TemplateApp {
    plot: Graphics,
    ctx: Option<egui::Context>,
    #[cfg(target_arch = "wasm32")]
    client: Option<wasm_sockets::EventClient>,
}

impl Default for TemplateApp {
    #[cfg(not(target_arch = "wasm32"))]
    fn default() -> Self {
        Self {
            plot: Graphics::new(),
            ctx: None,
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn default() -> Self {
        Self {
            plot: Graphics::new(),
            ctx: None,
            client: None,
        }
    }
}

impl TemplateApp {
    #[cfg(not(target_arch = "wasm32"))]
    fn start_read_ws(&mut self, ctx: &egui::Context) {
        if self.ctx.is_none() {
            self.ctx = Some(ctx.clone());
            // Start WebSocket processing thread
            tokio::runtime::Builder::new_current_thread()
                // .worker_threads(1)
                .enable_all()
                .build()
                .unwrap()
                .block_on(async {
                    tokio::time::sleep(Duration::from_secs(10)).await;
                    tracing::info!("Hello from async task");
                });
        }
    }

    #[cfg(target_arch = "wasm32")]
    /// WebSocket communication from WASM applicaiton
    fn start_read_ws(&mut self, ctx: &egui::Context) {
        tracing::debug!("Starting websocket commincation inside WASM");
        if self.client.is_none() {
            let mut client = wasm_sockets::EventClient::new("ws://127.0.0.1:8081/ws/").unwrap();
            client.set_on_error(Some(Box::new(|error| {
                tracing::error!("{:#?}", error);
            })));
            client.set_on_connection(Some(Box::new(|client: &wasm_sockets::EventClient| {
                tracing::info!("{:#?}", client.status);
                tracing::info!("Sending message...");
                client.send_string("Hello, World!").unwrap();
                client.send_binary(vec![20]).unwrap();
            })));
            client.set_on_close(Some(Box::new(|| {
                tracing::info!("Connection closed");
            })));
            client.set_on_message(Some(Box::new(
                |client: &wasm_sockets::EventClient, message: wasm_sockets::Message| {
                    tracing::info!("New Message: {:#?}", message);
                },
            )));
            self.client = Some(client);
        }
    }
}

impl epi::App for TemplateApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut epi::Frame) {
        if self.ctx.is_none() {
            self.ctx = Some(ctx.clone());
        }

        self.start_read_ws(ctx);

        let id = Id::new("Main");
        let available_rect = ctx.available_rect();
        let layer_id = LayerId::background(); //new(Order::Background, id);
        let clip_rect = ctx.input().screen_rect();
        let mut ui = Ui::new(ctx.clone(), layer_id, id, available_rect, clip_rect);

        self.plot.ui(&mut ui);

        egui::warn_if_debug_build(&mut ui);
    }

    fn max_size_points(&self) -> Vec2 {
        Vec2::new(f32::INFINITY, f32::INFINITY)
    }

    fn persist_native_window(&self) -> bool {
        false
    }

    fn persist_egui_memory(&self) -> bool {
        false
    }
}
