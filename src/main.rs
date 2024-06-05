use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tdlib::{
    enums::{AuthorizationState, Update, User},
    functions,
};
use tokio::sync::mpsc::{self, Receiver, Sender};

mod config;

fn ask_user(string: &str) -> String {
    tracing::info!("{}", string);
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

async fn handle_update(update: Update, auth_tx: &Sender<AuthorizationState>) {
    match update {
        Update::AuthorizationState(update) => {
            auth_tx.send(update.authorization_state).await.unwrap();
        },
        _ => (),
    }
}

async fn handle_authorization_state(
    client_id: i32,
    mut auth_rx: Receiver<AuthorizationState>,
    run_flag: Arc<AtomicBool>,
) -> Receiver<AuthorizationState> {
    while let Some(state) = auth_rx.recv().await {
        match state {
            AuthorizationState::WaitTdlibParameters => {
                let response = functions::set_tdlib_parameters(
                    false,
                    "db".into(),
                    String::new(),
                    String::new(),
                    false,
                    false,
                    false,
                    false,
                    std::env::var("API_ID").unwrap().parse().unwrap(),
                    std::env::var("API_HASH").unwrap().into(),
                    "en".into(),
                    "Desktop".into(),
                    String::new(),
                    env!("CARGO_PKG_VERSION").into(),
                    false,
                    true,
                    client_id,
                )
                .await;

                if let Err(error) = response {
                    tracing::error!("{}", error.message);
                }
            }
            AuthorizationState::WaitPhoneNumber => loop {
                let input = ask_user("请输入你的手机号 (包含国家代码，例如: +86):");
                let response =
                    functions::set_authentication_phone_number(input, None, client_id).await;
                match response {
                    Ok(_) => break,
                    Err(e) => tracing::error!("{}", e.message),
                }
            },
            AuthorizationState::WaitCode(_) => loop {
                let input = ask_user("输入验证码:");
                let response = functions::check_authentication_code(input, client_id).await;
                match response {
                    Ok(_) => break,
                    Err(e) => tracing::error!("{}", e.message),
                }
            },
            AuthorizationState::Ready => {
                break;
            }
            AuthorizationState::Closed => {
                // Set the flag to false to stop receiving updates from the
                // spawned task
                run_flag.store(false, Ordering::Release);
                break;
            }
            _ => (),
        }
    }

    auth_rx
}

#[tokio::main]
async fn main() {
    config::log::init();

    let api_id = std::env::var("API_ID");
    let api_hash = std::env::var("API_HASH");
    if api_id.is_err() || api_hash.is_err() {
        panic!("请正确配置 API_ID 和 API_HASH 环境变量，当前 API_ID: {:?} API_HASH: {:?}", api_id, api_hash);
    }

    // Create the client object
    let client_id = tdlib::create_client();

    // Create a mpsc channel for handling AuthorizationState updates separately
    // from the task
    let (auth_tx, auth_rx) = mpsc::channel(5);

    // Create a flag to make it possible to stop receiving updates
    let run_flag = Arc::new(AtomicBool::new(true));
    let run_flag_clone = run_flag.clone();

    // Spawn a task to receive updates/responses
    let handle = tokio::spawn(async move {
        while run_flag_clone.load(Ordering::Acquire) {
            if let Some((update, _client_id)) = tdlib::receive() {
                handle_update(update, &auth_tx).await;
            }
        }
    });

    // Set a fairly low verbosity level. We mainly do this because tdlib
    // requires to perform a random request with the client to start receiving
    // updates for it.
    functions::set_log_verbosity_level(2, client_id)
        .await
        .unwrap();

    // Handle the authorization state to authenticate the client
    let auth_rx = handle_authorization_state(client_id, auth_rx, run_flag.clone()).await;

    // Run the get_me() method to get user information
    let User::User(me) = functions::get_me(client_id).await.unwrap();
    tracing::info!("Hi, I'm {}", me.first_name);

    let chats = functions::get_chats(None, 20, client_id).await;
    if chats.is_err() {
        tracing::error!("获取前二十个聊天列表失败: {:?}", chats.as_ref().err())
    }
    let tdlib::enums::Chats::Chats(chats) = chats.unwrap();
    for chat_id in chats.chat_ids {
        let chat = functions::get_chat(chat_id, client_id).await;
        if chat.is_err() {
            tracing::error!("获取 id 为 {} 的聊天失败: {:?}", chat_id, chat.as_ref().err())
        }
        let tdlib::enums::Chat::Chat(chat) = chat.unwrap();
        tracing::info!("title: {} id: {}", chat.title, chat.id);
    }

    let akile_chat_id = std::env::var("AKILE_CHAT_ID");
    if akile_chat_id.is_ok() {
        let akile_chat_id = akile_chat_id.unwrap().parse().unwrap();
        functions::open_chat(akile_chat_id, client_id).await.unwrap();
        let message = functions::send_message(akile_chat_id, 0, None, None, None,
            tdlib::enums::InputMessageContent::InputMessageText(tdlib::types::InputMessageText {
                text: tdlib::types::FormattedText {
                    text: "/checkin@akilecloud_bot".to_string(),
                    entities: vec![]
                },
                disable_web_page_preview: true,
                clear_draft: true
            }), client_id).await;
        if message.is_err() {
            tracing::error!("发送 akile 签到失败: {:?}", message.as_ref().err())
        }
        let tdlib::enums::Message::Message(message) = message.unwrap();
        tracing::info!("message id: {} content: {:?}", message.id, message.content);
    } else {
        tracing::info!("AKILE_CHAT_ID 环境变量配置错误，跳过 akile 签到， AKILE_CHAT_ID: {:?}", akile_chat_id);
    }
    


    // Tell the client to close
    functions::close(client_id).await.unwrap();

    // Handle the authorization state to wait for the "Closed" state
    handle_authorization_state(client_id, auth_rx, run_flag.clone()).await;

    // Wait for the previously spawned task to end the execution
    handle.await.unwrap();
}