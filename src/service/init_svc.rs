use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tdlib::{
    enums::{AuthorizationState, Update},
    functions,
};
use tokio::sync::{mpsc::{self, Receiver, Sender}, RwLock};

async fn handle_update(update: Update, auth_tx: &Sender<AuthorizationState>) {
    match update {
        Update::AuthorizationState(update) => {
            auth_tx.send(update.authorization_state).await.unwrap();
        },
        _ => (),
    }
}

#[derive(Debug, Clone)]
pub struct InitData {
    pub client_id: i32,
    pub auth_rx: Arc<RwLock<Receiver<AuthorizationState>>>,
    pub run_flag: Arc<RwLock<AtomicBool>>,
}

pub async fn init() -> InitData {
    // Create the client object
    let client_id = tdlib::create_client();

    // Create a mpsc channel for handling AuthorizationState updates separately
    // from the task
    let (auth_tx, auth_rx) = mpsc::channel(5);

    // Create a flag to make it possible to stop receiving updates
    let run_flag = Arc::new(RwLock::new(AtomicBool::new(true)));
    let run_flag_clone = run_flag.clone();

    // Spawn a task to receive updates/responses
    tokio::spawn(async move {
        while run_flag_clone.read().await.load(Ordering::Acquire) {
            if let Some((update, _client_id)) = tdlib::receive() {
                handle_update(update, &auth_tx).await;
            }
        }
    });

    // Set a fairly low verbosity level. We mainly do this because tdlib
    // requires to perform a random request with the client to start receiving
    // updates for it.
    functions::set_log_verbosity_level(2, client_id).await.unwrap();

    InitData {
        client_id,
        auth_rx: Arc::new(RwLock::new(auth_rx)),
        run_flag,
    }
}