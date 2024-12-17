use std::{collections::HashMap, sync::Arc};

use tokio::sync::{
    mpsc::{self},
    Mutex,
};
use uuid::Uuid;

pub type Frame = String;
type UserDimensions = Arc<Mutex<HashMap<Uuid, (u16, u16)>>>; // Stores dimensions per client UUID
type ClientChannels = Arc<Mutex<HashMap<Uuid, mpsc::Sender<Frame>>>>; // Channels to send TUI frames
type CloseFrameSender = Arc<Mutex<HashMap<Uuid, bool>>>;
#[derive(Clone, Debug)]
pub struct AppState {
    pub client_tui_tx: ClientChannels,
    pub user_dimensions: UserDimensions,
    pub close_frame_sender: CloseFrameSender,
}
