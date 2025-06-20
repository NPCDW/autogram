use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tdlib_rs::{
    enums::{AuthorizationState, MessageContent, Update},
    types::{UpdateNewMessage, UpdateMessageContent},
    functions
};
use tokio::sync::{mpsc::{self, Receiver, Sender}, RwLock};

use super::auth_svc;

pub struct SimpleMessage {
    pub id: i64,
    pub chat_id: i64,
    pub content: MessageContent,
}

async fn handle_update(update: Update, auth_tx: &Sender<AuthorizationState>, msg_tx: &Sender<(Option<UpdateNewMessage>, Option<UpdateMessageContent>)>) {
    match update {
        Update::AuthorizationState(update) => {
            auth_tx.send(update.authorization_state).await.unwrap();
        },
        Update::NewMessage(update) => {
            msg_tx.send((Some(update), None)).await.unwrap_or_else(|e| {
                tracing::error!("发送消息失败： {:?}", e);
            });
        },
        Update::MessageContent(update) => {
            msg_tx.send((None, Some(update))).await.unwrap_or_else(|e| {
                tracing::error!("发送消息失败： {:?}", e);
            });
        },
        _ => (),
    }
}

#[derive(Debug, Clone)]
pub struct InitData {
    pub client_id: i32,
    pub auth_rx: Arc<RwLock<Receiver<AuthorizationState>>>,
    pub msg_rx: Arc<RwLock<Receiver<(Option<UpdateNewMessage>, Option<UpdateMessageContent>)>>>,
    pub run_flag: Arc<RwLock<AtomicBool>>,
    pub bot_token: Option<String>,
}

pub async fn init(already_login: bool) -> InitData {
    // Create the client object
    let client_id = tdlib_rs::create_client();

    // Create a mpsc channel for handling AuthorizationState updates separately
    // from the task
    let (auth_tx, auth_rx) = mpsc::channel(5);
    let (msg_tx, msg_rx) = mpsc::channel(500);

    // Create a flag to make it possible to stop receiving updates
    let run_flag = Arc::new(RwLock::new(AtomicBool::new(true)));
    let run_flag_clone = run_flag.clone();

    // Spawn a task to receive updates/responses
    tokio::spawn(async move {
        while run_flag_clone.read().await.load(Ordering::Acquire) {
            if let Some((update, _client_id)) = tdlib_rs::receive() {
                handle_update(update, &auth_tx, &msg_tx).await;
            }
        }
    });

    // Set a fairly low verbosity level. We mainly do this because tdlib
    // requires to perform a random request with the client to start receiving
    // updates for it.
    functions::set_log_verbosity_level(2, client_id).await.unwrap();

    // Handle the authorization state to authenticate the client
    let auth_rx = Arc::new(RwLock::new(auth_rx));
    if already_login {
        auth_svc::handle_already_login_state(client_id, auth_rx.clone()).await;
    } else {
        auth_svc::handle_authorization_state(client_id, auth_rx.clone(), run_flag.clone()).await;
    }

    let bot_token = std::env::var("BOT_TOKEN");
    if bot_token.is_err() {
        tracing::debug!("未配置 BOT_TOKEN 环境变量");
    }

    InitData {
        client_id,
        auth_rx,
        msg_rx: Arc::new(RwLock::new(msg_rx)),
        run_flag,
        bot_token: bot_token.ok(),
    }
}

pub async fn close(inti_data: InitData) {
    // Tell the client to close
    functions::close(inti_data.client_id).await.unwrap();

    // Handle the authorization state to wait for the "Closed" state
    auth_svc::handle_authorization_state(inti_data.client_id, inti_data.auth_rx.clone(), inti_data.run_flag.clone()).await;
}