use std::{
    collections::HashMap,
    fs::{self, File},
    io::Write,
    sync::Arc,
    time::Duration,
};

use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::{Html, IntoResponse},
    routing::{get, get_service},
    Router,
};
use ratatui::buffer;
use serde_json::Value;
use tokio::sync::{broadcast, Mutex};
use tower_http::services::ServeDir;
use uuid::Uuid;

use tui_2::init;

mod parse_frame;
mod tui_2;

type Frame = String;
type TerminalState = Arc<Mutex<HashMap<Uuid, String>>>;

#[derive(Clone, Debug)]
struct AppState {
    tx: broadcast::Sender<Frame>,
    terminals: TerminalState,
}
#[tokio::main]
async fn main() {
    let (tx, _) = broadcast::channel::<Frame>(1);
    let app_state = AppState {
        tx: tx.clone(),
        terminals: Arc::new(Mutex::new(HashMap::new())),
    };
    //http://192.168.1.214:3000/
    //TODO make it so other devices have their own tui not just one shared one
    let router = Router::new()
        .route("/", get(root_get))
        .route("/ws", get(ui_websocket))
        .nest_service("/assets", get_service(ServeDir::new("assets")))
        .nest_service("/images", get_service(ServeDir::new("images")))
        .with_state(app_state.clone());

    // Init terminal
    // init_tui();
    tokio::task::spawn_blocking(move || loop {
        let frame = read_buffer_with_retry_to_json();
        let _ = tx.send(frame);
        let seconds = Duration::from_millis(100);
        std::thread::sleep(seconds);
    });

    let listener = tokio::net::TcpListener::bind("192.168.1.213:3001")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, router).await.unwrap();
}

#[axum::debug_handler]
async fn root_get() -> impl IntoResponse {
    let markup = tokio::fs::read_to_string("assets/index.html")
        .await
        .unwrap();

    Html(markup)
}

#[axum::debug_handler]
async fn ui_websocket(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(|ws: WebSocket| async { realtime_websocket_stream(state, ws).await })
}

async fn realtime_websocket_stream(app_state: AppState, mut ws: WebSocket) {
    let mut rx = app_state.tx.subscribe();

    loop {
        tokio::select! {
            // Receive messages from the broadcast channel
            Ok(msg) = rx.recv() => {
                ws.send(Message::Text(serde_json::to_string(&msg).unwrap()))
                    .await
                    .expect("Failed to send frame");
            }
            // Handle messages from the WebSocket
            Some(Ok(Message::Text(text))) = ws.recv() => {
                if let Ok(value) = serde_json::from_str::<Value>(&text) {
                    if let Some(key) = value.get("key") {
                        log_from_socket(key.as_str().unwrap(), "key_log.txt");
                    }
                    if let (Some(width), Some(height)) = (value.get("width"), value.get("height")) {
                        let width = width.as_u64().and_then(|v| v.try_into().ok()).unwrap();
                        let height = height.as_u64().and_then(|v| v.try_into().ok()).unwrap();
                        tokio::task::spawn_blocking(move || {
                            let _ = init(width, height);
                        });

                    }
                }
            }
            // Handle WebSocket close or error
            else => break,
        }
    }
}

fn log_from_socket(key: &str, path_name: &str) {
    let mut file = File::create(path_name).expect("Failed to create or open the file");
    file.write_all(format!("{}", key).as_bytes())
        .expect("Failed to write to file");
}

fn read_buffer_with_retry_to_json() -> String {
    let mut file_contents = String::new();

    while file_contents.is_empty() {
        file_contents =
            fs::read_to_string("output.txt").expect("Should have been able to read the file");

        if file_contents.is_empty() {
            println!("File is empty, retrying...");
            std::thread::sleep(Duration::from_millis(10)); // Adjust time as needed
        }
    }

    // Once the file is not empty, parse the buffer
    let buffer = parse_frame::Buffer::from_string(&file_contents);
    serde_json::to_string(&buffer)
        .expect("Serde Json TO string error")
}
