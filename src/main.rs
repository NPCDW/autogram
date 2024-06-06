use clap::Parser;
use config::args_conf::AppCommand;

mod config;
mod service;

#[tokio::main]
async fn main() {
    config::log_conf::init();

    let api_id = std::env::var("API_ID");
    let api_hash = std::env::var("API_HASH");
    if api_id.is_err() || api_hash.is_err() {
        panic!("请正确配置 API_ID 和 API_HASH 环境变量，当前 API_ID: {:?} API_HASH: {:?}", api_id, api_hash);
    }

    let args = config::args_conf::Args::parse();

    let inti_data = service::init_svc::init().await;

    service::signal_svc::handle(inti_data.clone());

    match args.command {
        AppCommand::Login(login_args) => {
            service::login_svc::login(inti_data.clone(), login_args).await;
        },
        AppCommand::Start => {
            let res = service::hello_svc::hello(inti_data.client_id).await;
            if res.is_err() {
                return;
            }
            service::akile_svc::checkin(inti_data.client_id).await;
        },
    }
}