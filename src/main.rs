use clap::Parser;
use config::args_conf::AppCommand;

mod config;
mod service;
mod util;

#[tokio::main]
async fn main() {
    config::log_conf::init();

    service::signal_svc::handle();

    let api_id = std::env::var("API_ID");
    let api_hash = std::env::var("API_HASH");
    if api_id.is_err() || api_hash.is_err() {
        panic!("请正确配置 API_ID 和 API_HASH 环境变量，当前 API_ID: {:?} API_HASH: {:?}", api_id, api_hash);
    }

    let args = config::args_conf::Args::parse();

    let init_data = match args.command {
        AppCommand::Login => service::init_svc::init(false).await,
        _ => service::init_svc::init(true).await,
    };

    match args.command {
        AppCommand::Chats(param) => {
            let res = service::chats_svc::top(init_data.client_id, param).await;
            if res.is_err() {
                tracing::error!("获取聊天列表失败: {:?}", res.err());
            }
        },
        AppCommand::Chat(param) => {
            let res = service::chat_svc::chat(init_data.clone(), param).await;
            if res.is_err() {
                tracing::error!("发送聊天消息失败: {:?}", res.err());
            }
        },
        AppCommand::MultiChat(param) => {
            for chat in param.chat {
                let chat = serde_json::from_str(&chat);
                if chat.is_err() {
                    tracing::error!("解析聊天参数失败: {:?}", chat.err());
                    continue;
                }
                let res = service::chat_svc::chat(init_data.clone(), chat.unwrap()).await;
                if res.is_err() {
                    tracing::error!("发送多个聊天消息失败: {:?}", res.err());
                    continue;
                }
            }
        },
        AppCommand::Listen(param) => {
            let res = service::listen_svc::listen(init_data.clone(), param).await;
            if res.is_err() {
                tracing::error!("监听聊天失败: {:?}", res.err());
            }
        },
        AppCommand::MultiListen(param) => {
            let res = service::multi_listen_svc::listen(init_data.clone(), param).await;
            if res.is_err() {
                tracing::error!("监听多个聊天失败: {:?}", res.err());
            }
        },
        AppCommand::Follow(param) => {
            let res = service::follow_svc::follow(init_data.clone(), param).await;
            if res.is_err() {
                tracing::error!("关注用户动向失败: {:?}", res.err());
            }
        },
        AppCommand::GuessCode(param) => {
            let res = service::guess_code_svc::guess_code(init_data.clone(), param).await;
            if res.is_err() {
                tracing::error!("关注用户动向失败: {:?}", res.err());
            }
        },
        AppCommand::RedPacket(param) => {
            let res = service::red_packet_svc::grab(init_data.clone(), param).await;
            if res.is_err() {
                tracing::error!("抢红包失败: {:?}", res.err());
            }
        },
        AppCommand::CreateAccount(param) => {
            let res = service::create_account_svc::create(init_data.clone(), param).await;
            if res.is_err() {
                tracing::error!("创建账户失败: {:?}", res.err());
            }
        },
        _ => (),
    }
    service::init_svc::close(init_data).await;
}