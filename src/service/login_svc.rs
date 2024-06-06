use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tdlib::{
    enums::AuthorizationState,
    functions,
};
use tokio::sync::{mpsc::Receiver, RwLock};

use crate::config::args_conf::LoginArgs;

use super::init_svc::InitData;

fn ask_user(string: &str) -> String {
    tracing::info!("{}", string);
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

async fn handle_authorization_state(
    client_id: i32,
    auth_rx: Arc<RwLock<Receiver<AuthorizationState>>>,
    run_flag: Arc<RwLock<AtomicBool>>,
    phone: Option<String>
) {
    while let Some(state) = auth_rx.write().await.recv().await {
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
                let phone_number = if phone.as_ref().is_some() {
                    phone.clone().unwrap()
                } else {
                    ask_user("请输入你的手机号 (包含国家代码，例如: +86):")
                };
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

pub async fn login(inti_data: InitData, login_args: LoginArgs) {
    // Handle the authorization state to authenticate the client
    handle_authorization_state(inti_data.client_id,inti_data.auth_rx, inti_data.run_flag.clone(), login_args.phone).await;
}

pub async fn logout(inti_data: InitData) {
    // Tell the client to close
    functions::close(inti_data.client_id).await.unwrap();

    // Handle the authorization state to wait for the "Closed" state
    handle_authorization_state(inti_data.client_id, inti_data.auth_rx, inti_data.run_flag.clone(), None).await;
}