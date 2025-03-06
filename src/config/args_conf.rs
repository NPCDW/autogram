use clap::{Parser, Subcommand};
use serde::Deserialize;

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
    /// 向多个聊天发送消息
    #[command()]
    MultiChat(MultiChatArgs),
    /// 监听一个聊天
    #[command()]
    Listen(ListenArgs),
    /// 监听多个聊天
    #[command()]
    MultiListen(MultiListenArgs),
    /// 关注一个用户
    #[command()]
    Follow(FollowArgs),
}

#[derive(Parser)]
pub struct ChatsArgs {
    /// 查询前多少个聊天
    #[arg(short, long, default_value_t = 20)]
    pub top: i32,
    /// 是否归档中的聊天
    #[arg(short, long)]
    pub archive: bool,
}

#[derive(Parser, Deserialize, Clone)]
pub struct ChatArgs {
    /// 聊天ID，请使用 --chat-id='-123456789' 勿使用 --chat-id '-123456789'
    #[arg(short, long)]
    pub chat_id: i64,
    /// 聊天消息内容
    #[arg(short, long)]
    pub message: String,
    /// 是否归档中的聊天
    #[arg(short, long)]
    pub archive: Option<bool>,
    /// 在回复的消息中点击按钮
    #[arg(long)]
    pub type_button: Option<String>,
    /// 转发回复的第一条消息，防止对方删除，聊天ID
    #[arg(long)]
    pub forward_chat_id: Option<i64>,
    /// 转发回复的第一条消息，防止对方删除，主题ID
    #[arg(long)]
    pub forward_topic_id: Option<i64>,
}

#[derive(Parser)]
pub struct MultiChatArgs {
    /// 聊天ID，请使用 --chat-id='-123456789' 勿使用 --chat-id '-123456789'
    #[arg(short, long)]
    pub chat: Vec<String>,
}

#[derive(Parser)]
pub struct ListenArgs {
    /// 聊天ID，请使用 --chat-id='-123456789' 勿使用 --chat-id '-123456789'
    #[arg(short, long)]
    pub chat_id: i64,
    /// 是否归档中的聊天
    #[arg(short, long)]
    pub archive: bool,
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
pub struct MultiListenArgs {
    /// 聊天ID，请使用 --chat-id='-123456789' 勿使用 --chat-id '-123456789'
    #[arg(short, long)]
    pub chat_id: Vec<i64>,
    /// 是否归档中的聊天
    #[arg(short, long)]
    pub archive: bool,
    /// 回调地址
    #[arg(long)]
    pub webhook_url: String,
}

#[derive(Parser)]
pub struct FollowArgs {
    /// 用户ID，请使用 --user-id='-123456789' 勿使用 --user-id '-123456789'
    #[arg(short, long)]
    pub user_id: Vec<i64>,
    /// 转发到的聊天ID，请使用 --forward-chat-id='-123456789' 勿使用 --forward-chat-id '-123456789'
    #[arg(long)]
    pub forward_chat_id: Option<i64>,
    /// 转发到的聊天的主题ID，请使用 --forward-topic-id='-123456789' 勿使用 --forward-topic-id '-123456789'
    #[arg(long)]
    pub forward_topic_id: Option<i64>,
    /// 是否归档中的聊天
    #[arg(short, long)]
    pub forward_chat_archive: bool,
    /// 回调地址
    #[arg(long)]
    pub webhook_url: Option<String>,
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: AppCommand,
}