use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tdlib::{
    enums::{AuthorizationState, User},
    functions,
};
use tokio::sync::{mpsc::Receiver, RwLock};

fn ask_user(string: &str) -> String {
    tracing::info!("{}", string);
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
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

pub async fn handle_already_login_state(
    client_id: i32,
    auth_rx: Arc<RwLock<Receiver<AuthorizationState>>>,
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

pub async fn handle_authorization_state(
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
