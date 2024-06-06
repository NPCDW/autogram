use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tdlib::{
    enums::{AuthorizationState, Update, User},
    functions,
};
use tokio::sync::{mpsc::{self, Receiver, Sender}, RwLock};

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

async fn set_tdlib_param(client_id: i32) {
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

async fn handle_already_login_state(
    client_id: i32,
    auth_rx: Arc<RwLock<Receiver<AuthorizationState>>>
) {
    while let Some(state) = auth_rx.write().await.recv().await {
        match state {
            AuthorizationState::WaitTdlibParameters => {
                set_tdlib_param(client_id).await;
            }
            AuthorizationState::Ready => {
                let User::User(me) = functions::get_me(client_id).await.unwrap();
                tracing::info!("欢迎回来 [{}]", me.first_name);
                break;
            }
            _ => {
                tracing::error!("未登录，请先登录");
                std::process::exit(0);
            },
        }
    }
}

async fn handle_authorization_state(
    client_id: i32,
    auth_rx: Arc<RwLock<Receiver<AuthorizationState>>>,
    run_flag: Arc<RwLock<AtomicBool>>
) {
    while let Some(state) = auth_rx.write().await.recv().await {
        match state {
            AuthorizationState::WaitTdlibParameters => {
                set_tdlib_param(client_id).await;
            }
            AuthorizationState::WaitPhoneNumber => loop {
                let phone_number = ask_user("请输入你的手机号 (包含国家代码，例如: +86):");
                let response =
                    functions::set_authentication_phone_number(phone_number, None, client_id).await;
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
                let User::User(me) = functions::get_me(client_id).await.unwrap();
                tracing::info!("Login successful [{}]", me.first_name);
                break;
            }
            AuthorizationState::Closed => {
                // Set the flag to false to stop receiving updates from the
                // spawned task
                run_flag.write().await.store(false, Ordering::Release);
                break;
            }
            _ => (),
        }
    }
}

#[derive(Debug, Clone)]
pub struct InitData {
    pub client_id: i32,
    pub auth_rx: Arc<RwLock<Receiver<AuthorizationState>>>,
    pub run_flag: Arc<RwLock<AtomicBool>>,
}

pub async fn init(already_login: bool) -> InitData {
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
    functions::set_log_verbosity_level(1, client_id).await.unwrap();

    // Handle the authorization state to authenticate the client
    let auth_rx = Arc::new(RwLock::new(auth_rx));
    if already_login {
        handle_already_login_state(client_id, auth_rx.clone()).await;
    } else {
        handle_authorization_state(client_id, auth_rx.clone(), run_flag.clone()).await;
    }

    InitData {
        client_id,
        auth_rx,
        run_flag,
    }
}

pub async fn logout(inti_data: InitData) {
    // Tell the client to close
    functions::close(inti_data.client_id).await.unwrap();

    // Handle the authorization state to wait for the "Closed" state
    handle_authorization_state(inti_data.client_id, inti_data.auth_rx, inti_data.run_flag.clone()).await;
}