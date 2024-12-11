use std::{fs::{self, File}, io::Write, time::Duration};

use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::{Html, IntoResponse},
    routing::{get, get_service},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::broadcast;
use tower_http::services::ServeDir;
use tui_2::init;

mod tui_2;

type Frame = String;

#[tokio::main]
async fn main() {
    let (tx, _) = broadcast::channel::<Frame>(1);


    let app_state = AppState { tx: tx.clone() };

    let router = Router::new()
        .route("/", get(root_get))
        .route("/ws", get(ui_websocket))
        .nest_service("/assets", get_service(ServeDir::new("assets")))
        .nest_service("/images", get_service(ServeDir::new("images")))
        .with_state(app_state.clone());

    // Init terminal
    // init_tui();
    //Draw frame every
    tokio::task::spawn_blocking(move || {
        init();
    });
    tokio::task::spawn_blocking(move || {
        loop {
            let frame = get_frame();
            let _ = tx.send(frame);
            let seconds = Duration::from_millis(100);
            std::thread::sleep(seconds);
        }
    });

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3001")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, router).await.unwrap();
}

#[derive(Clone)]
struct AppState {
    tx: broadcast::Sender<Frame>,
}

#[axum::debug_handler]
async fn root_get() -> impl IntoResponse {
    let markup = tokio::fs::read_to_string("assets/index.html").await.unwrap();

    Html(markup)
}

#[axum::debug_handler]
async fn ui_websocket(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|ws: WebSocket| async { realtime_websocket_stream(state, ws).await })
}

#[derive(Deserialize, Serialize, Debug)]
struct ResizeMessage {
    r#type: String,
    swidth: u16,
    sheight: u16,
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
                    
                    // Handle the "resize" type message
                    if let Some(r#type) = value.get("type") {
                        if r#type == "resize" {
                            // Try to deserialize the resize message
                            if let Ok(resize_message) = serde_json::from_str::<ResizeMessage>(&text) {
                                println!("Received resize message: {:?}", resize_message);
                                // Add your logic to handle the resize message here
                                let resize = format!("{},{}",resize_message.swidth,resize_message.sheight);
                                println!("{:?}", resize);

                                log_from_socket(&resize, "resize.txt");
                                // For example, adjusting the display size, etc.
                            }
                        }
                    }

                    // Handle other keys or messages
                    if let Some(key) = value.get("key") {
                        println!("Key pressed: {}", key.as_str().unwrap_or("Unknown key"));
                        log_from_socket(key.as_str().unwrap(), "key_log.txt");
                        // Add your logic here to handle the key input
                    }
                }
            }
            // Handle WebSocket close or error
            else => break,
        }
    }
}


// async fn realtime_websocket_stream(app_state: AppState, mut ws: WebSocket) {
//     let mut rx = app_state.tx.subscribe();

//     loop {
//         tokio::select! {
//             // Receive messages from the broadcast channel
//             Ok(msg) = rx.recv() => {
//                 ws.send(Message::Text(serde_json::to_string(&msg).unwrap()))
//                     .await
//                     .expect("Failed to send frame");
//             }
//             // Handle messages from the WebSocket
//             Some(Ok(Message::Text(text))) = ws.recv() => {
//                 if let Ok(value) = serde_json::from_str::<Value>(&text) {
//                     println!("{:?}",value);
//                     if let Some(key) = value.get("key") {
//                         println!("Key pressed: {}", key.as_str().unwrap_or("Unknown key"));
//                         log_from_socket(key.as_str().unwrap(),"key_log.txt");
//                         // Add your logic here to handle the key input
//                     }
//                 }
//             }
//             // Handle WebSocket close or error
//             else => break,
//         }
//     }
// }
fn log_from_socket(key: &str, path_name: &str) {
    println!("writing file");
    let mut file = File::create(path_name).expect("Failed to create or open the file");
    file.write_all( format!("{}", key).as_bytes()).expect("Failed to write to file");
}

// async fn realtime_frames_stream(app_state: AppState, mut ws: WebSocket) {
//     let mut rx = app_state.tx.subscribe();

//     while let Ok(msg) = rx.recv().await {
//         ws.send(Message::Text(serde_json::to_string(&msg).unwrap()))
//             .await
//             .expect("WTF");
//     }
// }

fn get_frame() -> String {
    let contents = fs::read_to_string("output.txt")
        .expect("Should have been able to read the file");
    contents //output.txt
}
