use std::process;
use tokio::signal::ctrl_c;
use tokio::signal::unix::{signal, SignalKind};

pub fn handle() {
    tokio::spawn(async {
        let mut sig = signal(SignalKind::terminate()).expect("failed to listen for SIGTERM signal");
        sig.recv().await;
        tracing::info!("got SIGTERM signal. autogram is exiting");
        process::exit(0);
    });
    
    tokio::spawn(async {
        ctrl_c().await.expect("failed to listen for ctrl_c event");
        tracing::info!("got CTRL-C. autogram is exiting");
        process::exit(0);
    });
}