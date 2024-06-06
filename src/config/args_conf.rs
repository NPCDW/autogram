use clap::{Parser, Subcommand};

#[derive(Subcommand)]
pub enum AppCommand {
    #[command()]
    Login(LoginArgs),
    #[command()]
    Start,
}

#[derive(Parser)]
pub struct LoginArgs {
    #[arg(short, long)]
    pub phone: Option<String>,
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: AppCommand,
}