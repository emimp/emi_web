use std::{collections::HashMap, fs::{self, File}, io::Write, sync::Arc, time::Duration};

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
use tokio::sync::{broadcast, Mutex};
use tower_http::services::ServeDir;

use tui_2::init;
use uuid::Uuid;

mod tui_2;

type Frame = String;
type TerminalState = Arc<Mutex<HashMap<Uuid, String>>>;

#[tokio::main]
async fn main() {
    let (tx, _) = broadcast::channel::<Frame>(1);
    let app_state = AppState {
        tx: tx.clone(),
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
    tokio::task::spawn_blocking(move || {
        let _ = init(22,22);
    });
    

    tokio::task::spawn_blocking(move || {
        loop {
            let frame = get_frame();
            let _ = tx.send(frame);
            let seconds = Duration::from_millis(100);
            std::thread::sleep(seconds);
        }
    });

    let listener = tokio::net::TcpListener::bind("192.168.1.214:3000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, router).await.unwrap();
}
#[derive(Clone, Debug)]
struct AppState {
    tx: broadcast::Sender<Frame>,
    terminals: TerminalState,
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
                    if let (Some(swidth), Some(sheight)) = (value.get("swidth"), value.get("sheight")) {
                        let resize = format!("{},{}", swidth, sheight);
                        println!("resize: {}", resize);
                        log_from_socket(&resize, "resize.txt");
                    }

                    // Handle other keys or messages
                    if let Some(key) = value.get("key") {
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
//     let init_thread = Arc::new(Mutex::new(None::<tokio::task::JoinHandle<()>>));

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
//                     if let (Some(swidth), Some(sheight)) = (value.get("swidth"), value.get("sheight")) {
//                         let resize = format!("{},{}", swidth, sheight);
//                         println!("resize: {}", resize);
//                         log_from_socket(&resize, "resize.txt");

//                         // Check if there is an old thread for init and remove it
//                         let mut thread_lock = init_thread.lock().await;
//                         if let Some(old_handle) = thread_lock.take() {
//                             old_handle.abort(); // Cancel the previous thread if any
//                             println!("Removed old init thread.");
//                         }

//                         // Spawn a new thread for handling init-related tasks
//                         let new_handle = tokio::spawn(async move {
//                             // Add your logic for the "init" thread here
//                             let (w,h) = resize.split_once(',').unwrap();
//                             let _ = init(w.parse().unwrap(), h.parse().unwrap());

//                             println!("Started new init thread for resize: {}", resize);
//                             // Simulate some processing here if needed
//                         });

//                         // Store the handle to the new thread
//                         *thread_lock = Some(new_handle);
//                     }

//                     // Handle other keys or messages
//                     if let Some(key) = value.get("key") {
//                         println!("Key pressed: {}", key.as_str().unwrap_or("Unknown key"));
//                         log_from_socket(key.as_str().unwrap(), "key_log.txt");
//                         // Add your logic here to handle the key input
//                     }
//                 }
//             }
//             // Handle WebSocket close or error
//             else => break,
//         }
//     }
// }
////olddddd but mauube
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
//                     if let (Some(swidth), Some(sheight)) = (value.get("swidth"), value.get("sheight")) {
//                         let resize = format!("{},{}", swidth, sheight);
//                         println!("resize: {}", resize);
//                         log_from_socket(&resize, "resize.txt");
                        
//                     }
                    

//                     // Handle other keys or messages
//                     if let Some(key) = value.get("key") {
//                         println!("Key pressed: {}", key.as_str().unwrap_or("Unknown key"));
//                         log_from_socket(key.as_str().unwrap(), "key_log.txt");
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
