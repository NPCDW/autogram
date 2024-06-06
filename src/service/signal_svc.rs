use std::process;
use tokio::signal::ctrl_c;
use tokio::signal::unix::{signal, SignalKind};

use super::init_svc::InitData;

pub fn handle(inti_data: InitData) {
    let inti_data_clone = inti_data.clone();
    tokio::spawn(async {
        let mut sig = signal(SignalKind::terminate()).expect("failed to listen for SIGTERM signal");
        sig.recv().await;
        tracing::info!("got SIGTERM signal. autogram is exiting");
        super::login_svc::logout(inti_data_clone).await;
        process::exit(0);
    });
    
    tokio::spawn(async {
        ctrl_c().await.expect("failed to listen for ctrl_c event");
        tracing::info!("got CTRL-C. autogram is exiting");
        super::login_svc::logout(inti_data).await;
        process::exit(0);
    });
}