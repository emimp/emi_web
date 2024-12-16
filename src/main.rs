use std::{collections::HashMap, fs::File, sync::Arc, thread::spawn, time::Duration};

use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::{Html, IntoResponse},
    routing::{get, get_service},
    Router,
};
use serde_json::Value;
use tokio::sync::{
    mpsc::{self},
    Mutex,
};
use tower_http::services::ServeDir;
use uuid::Uuid;

use tui::tui_init;

mod parse_frame;
mod tui;

type Frame = String;
type UserDimensions = Arc<Mutex<HashMap<Uuid, (u16, u16)>>>; // Stores dimensions per client UUID
type ClientChannels = Arc<Mutex<HashMap<Uuid, mpsc::Sender<Frame>>>>; // Channels to send TUI frames
type CloseFrameSender = Arc<Mutex<HashMap<Uuid, bool>>>;
#[derive(Clone, Debug)]
struct AppState {
    client_tui_tx: ClientChannels,
    user_dimensions: UserDimensions,
    close_frame_sender: CloseFrameSender
}

#[tokio::main]
async fn main() {
    let app_state = AppState {
        client_tui_tx: Arc::new(Mutex::new(HashMap::new())),
        user_dimensions: Arc::new(Mutex::new(HashMap::new())),
        close_frame_sender: Arc::new(Mutex::new(HashMap::new())),
    };

    let router = Router::new()
        .route("/", get(root_get))
        .route("/ws", get(ui_websocket))
        .nest_service("/assets", get_service(ServeDir::new("assets")))
        .nest_service("/images", get_service(ServeDir::new("images")))
        .with_state(app_state.clone());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3001")
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
    ws.on_upgrade(|ws: WebSocket| async {
        let uuid = Uuid::new_v4();
        println!("New connection with UUID: {}", uuid);
        realtime_websocket_stream(state, ws, uuid).await
    })
}

async fn realtime_websocket_stream(app_state: AppState, mut ws: WebSocket, uuid: Uuid) {
    let (tx, mut rx) = mpsc::channel::<Frame>(10);

    {
        let mut client_tui_tx = app_state.client_tui_tx.lock().await;
        client_tui_tx.insert(uuid, tx);
    }

    loop {
        tokio::select! {
            Some(frame) = rx.recv() => {
                if let Err(e) = ws.send(Message::Text(frame)).await {
                    eprintln!("Failed to send frame to UUID {}: {}", uuid, e);
                    break;
                }
            }
            Some(Ok(Message::Text(text))) = ws.recv() => {
                if let Ok(value) = serde_json::from_str::<Value>(&text) {
                    if let (Some(width), Some(height)) = (value.get("width"), value.get("height")) {
                        let (width, height) = (
                            width.as_u64().unwrap_or_default() as u16,
                            height.as_u64().unwrap_or_default() as u16,
                        );

                        // Store dimensions
                        {
                            let mut dimensions = app_state.user_dimensions.lock().await;
                            dimensions.insert(uuid, (width, height));
                        }

                        // Initialize TUI
                        tokio::task::spawn(init_and_stream_tui(app_state.clone(), uuid, width, height));
                    }
                }
            }
            else => break,
        }
    }

    //TODO Cleanup after disconnect
    // tokio::spawn(async move {
    //     cleanup_after_disconnect(app_state.clone().into(), uuid).await;
    // });
    {
        println!("removing {}", &uuid);
        {
            let mut client_tui_tx = app_state.client_tui_tx.lock().await;
            client_tui_tx.remove(&uuid);
            println!("{:?}",client_tui_tx);

        }
        {
            let mut user_dimensions = app_state.user_dimensions.lock().await;
            user_dimensions.remove(&uuid);
            println!("{:?}",user_dimensions);

        }
        {
            let mut close_frame_sender = app_state.close_frame_sender.lock().await;
            close_frame_sender.insert(uuid, true);
            println!("{:?}", close_frame_sender)
        }
        println!("path");
        File::create(format!("temp/{uuid}.remove")).unwrap();

    }
}

async fn init_and_stream_tui(app_state: AppState, uuid: Uuid, width: u16, height: u16) {
    spawn(move || {
        let _ = tui_init(width, height, uuid.to_string());
    });

    let tui_output_file = format!("temp/{uuid}.output");

    loop {
        println!("RUNNING: {}",uuid);
        let frame = read_buffer_with_retry_to_json(&tui_output_file);
        let client_tx = {
            let client_tui_tx = app_state.client_tui_tx.lock().await;
            client_tui_tx.get(&uuid).cloned()
        };

        if let Some(tx) = client_tx {
            if tx.send(frame).await.is_err() {
                break;
            }
        }

        if *app_state.close_frame_sender.lock().await.get(&uuid).unwrap_or(&false) {
            break
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

fn read_buffer_with_retry_to_json(file_path: &str) -> String {
    use std::fs;

    let mut file_contents = String::new();

    while file_contents.is_empty() {
        file_contents = fs::read_to_string(file_path).unwrap_or_default();
        if file_contents.is_empty() {
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    let buffer = parse_frame::Buffer::from_string(&file_contents);
    serde_json::to_string(&buffer).expect("Failed to serialize buffer to JSON")
}
