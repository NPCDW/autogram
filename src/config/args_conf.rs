use clap::{Parser, Subcommand};

#[derive(Subcommand)]
pub enum AppCommand {
    #[command()]
    Login,
    #[command()]
    Chats(ChatsArgs),
    #[command()]
    Chat(ChatArgs),
    #[command()]
    Start,
}

#[derive(Parser)]
pub struct ChatsArgs {
    #[arg(short, long, default_value_t = 20)]
    pub top: i32,
}

#[derive(Parser)]
pub struct ChatArgs {
    #[arg(short, long)]
    pub chat_id: i64,
    #[arg(short, long)]
    pub message: String,
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: AppCommand,
}