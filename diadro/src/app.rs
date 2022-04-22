use crate::graph::Graphics;
use crate::ws::{MousePosition, RequestInfo, WsMessages};
use chrono::{DateTime, Duration, Utc};
use eframe::egui::{Id, LayerId, Ui, Vec2};
use eframe::{egui, epi};
use uuid::Uuid;
use {std::cell::RefCell, std::rc::Rc};

#[cfg(target_arch = "wasm32")]
use wasm_sockets::EventClient;

pub struct TemplateApp {
    #[allow(dead_code)]
    id: String,
    plot: Graphics,
    ctx: Option<egui::Context>,
    packet_start: Option<DateTime<Utc>>,
    packet: Vec<WsMessages>,
    #[cfg(target_arch = "wasm32")]
    client: Rc<RefCell<Option<EventClient>>>,
    incoming_messages: Rc<RefCell<Vec<WsMessages>>>,
}

impl Default for TemplateApp {
    #[cfg(not(target_arch = "wasm32"))]
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            plot: Graphics::new(),
            ctx: None,
            packet_start: None,
            packet: Default::default(),
            incoming_messages: Rc::new(RefCell::new(Default::default())),
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            plot: Graphics::new(),
            ctx: None,
            packet_start: None,
            packet: vec![],
            client: Rc::new(RefCell::new(None)),
            incoming_messages: Default::default(),
        }
    }
}

/// Implies web-socket communications
impl TemplateApp {
    #[cfg(not(target_arch = "wasm32"))]
    /// web-socket processing threaad for not desktop application
    fn start_read_ws(&mut self, ctx: &egui::Context) {
        use std::time::Duration;

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

    #[cfg(not(target_arch = "wasm32"))]
    /// Send web socket message for Desctop application
    fn send(&self, _message: &str) {
        todo!();
    }

    #[cfg(target_arch = "wasm32")]
    /// Send web-socket message for WASM application
    fn send(&self, message: &str) {
        if let Some(client) = self.client.borrow().as_ref() {
            match client.send_string(message) {
                Ok(_) => tracing::debug!("WebSocket message sent successfully"),
                Err(err) => tracing::error!("Error sending ws message: {:?}", err),
            }
        }
    }

    /// Send message to web-socket using buffer
    /// Works in both desktop and WASM build because update in egui called enough offen to
    /// avoid special timers to send buffered data
    fn send_buffered(&mut self, message: WsMessages) {
        match self.packet_start {
            None => {
                self.packet.push(message);
                self.packet_start = Some(Utc::now());
            }
            Some(start) if Utc::now() - start > Duration::microseconds(100) => {
                match serde_json::to_string(&self.packet) {
                    Ok(msg) => {
                        tracing::debug!("Sending messages");
                        self.send(&msg);
                    }
                    Err(err) => tracing::error!("Error serializing messages: {:?}", err),
                }

                self.packet.clear();
                self.packet.push(message);
                let _ = self.packet_start.take();
            }
            _ => self.packet.push(message),
        }
    }

    #[cfg(target_arch = "wasm32")]
    /// WebSocket communication from WASM applicaiton
    fn start_read_ws(&mut self, _: &egui::Context) {
        if self.client.borrow().is_none() {
            tracing::debug!("Starting websocket commincation inside WASM");

            let window = match web_sys::window() {
                Some(wnd) => format!("ws://{}:{}/ws/{}", 
                    wnd.location().hostname().unwrap(), 
                    wnd.location().port().unwrap(),
                    self.id
                ),
                None => format!("ws://127.0.0.1:8081/ws/{}", self.id),
            };

            tracing::info!("WS location: {}", &window);

            let mut client = wasm_sockets::EventClient::new(&window).unwrap();
            client.set_on_error(Some(Box::new(|error| {
                tracing::error!("{:#?}", error);
            })));

            client.set_on_connection(Some(Box::new(|client: &wasm_sockets::EventClient| {
                tracing::info!("{:#?}", client.status);
                tracing::info!("Sending message...");
                client.send_string("Hello, World!").unwrap();
            })));

            let clone_cl = self.client.clone();
            client.set_on_close(Some(Box::new(move || {
                clone_cl.replace(None);
                tracing::info!("Connection closed");
            })));

            let incoming_messages = self.incoming_messages.clone();
            client.set_on_message(Some(Box::new(
                move |client: &wasm_sockets::EventClient, message: wasm_sockets::Message| {
                    match message {
                        wasm_sockets::Message::Text(text) => {
                            let m = text.trim();
                            match serde_json::from_str::<Vec<WsMessages>>(m) {
                                Ok(v) => incoming_messages.borrow_mut().extend(v),
                                Err(err) => tracing::error!("{}", err),
                            }
                        }
                        _ => tracing::error!("Unknown message incoming"),
                    }
                },
            )));

            self.client.replace(Some(client));
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

        // Send mouse position
        if let Some(pos) = ctx.input().pointer.hover_pos() {
            self.send_buffered(WsMessages::MousePosition(MousePosition {
                rq: RequestInfo {
                    board: "Main".to_string(),
                    user: self.id.clone(),
                },
                position: pos,
            }));
        }

        self.start_read_ws(ctx);

        let id = Id::new("Main");
        let available_rect = ctx.available_rect();
        let layer_id = LayerId::background();
        let clip_rect = ctx.input().screen_rect();
        let mut ui = Ui::new(ctx.clone(), layer_id, id, available_rect, clip_rect);

        // let incoming_message = self.incoming_messages.borrow();
        let msg = self.plot.ui(&mut ui, self.incoming_messages.borrow());
        match serde_json::to_string(&msg.inner) {
            Ok(s) => self.send(&s),
            Err(err) => tracing::error!("Error serializing: {}", err),
        }

        self.incoming_messages.borrow_mut().clear();
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
