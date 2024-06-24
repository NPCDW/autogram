use clap::{Parser, Subcommand};

#[derive(Subcommand)]
pub enum AppCommand {
    /// 登录，使用其他命令前，必须先登录
    #[command()]
    Login,
    /// 查看前几个聊天的ID和标题
    #[command()]
    Chats(ChatsArgs),
    /// 向一个聊天发送消息
    #[command()]
    Chat(ChatArgs),
    /// 监听一个聊天
    #[command()]
    Listen(ListenArgs),
}

#[derive(Parser)]
pub struct ChatsArgs {
    /// 查询前多少个聊天
    #[arg(short, long, default_value_t = 20)]
    pub top: i32,
}

#[derive(Parser)]
pub struct ChatArgs {
    /// 聊天ID，请使用 --chat-id='-123456789' 勿使用 --chat-id '-123456789'
    #[arg(short, long)]
    pub chat_id: i64,
    /// 聊天消息内容
    #[arg(short, long)]
    pub message: String,
}

#[derive(Parser)]
pub struct ListenArgs {
    /// 聊天ID，请使用 --chat-id='-123456789' 勿使用 --chat-id '-123456789'
    #[arg(short, long)]
    pub chat_id: i64,
    /// 是否查询历史消息
    #[arg(long)]
    pub history: bool,
    /// 最大查询的历史消息条数，可能会略微超出
    #[arg(long, default_value_t = 100)]
    pub max_history: usize,
    /// 回调地址
    #[arg(long)]
    pub webhook_url: String,
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: AppCommand,
}