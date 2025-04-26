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
    /// 猜码游戏
    #[command()]
    GuessCode(GuessCodeArgs),
    /// 抢红包游戏
    #[command()]
    RedPacket(RedPacketArgs),
    /// 创建账号（主动点击创建账户按钮尝试，但是几分钟后无法再点击，可能是 tg 限制，需要重启容器）
    #[command()]
    CreateAccount(CreateAccountArgs),
    /// 创建账号（监听群内机器人发送的开注消息，前往机器人注册）
    #[command()]
    CreateAccountListen(CreateAccountListenArgs),
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
pub struct GuessCodeArgs {
    /// 猜码机器人聊天ID
    #[arg(short, long)]
    pub chat_id: i64,
    /// 是否归档中的聊天
    #[arg(short, long)]
    pub archive: Option<bool>,
    /// 邀请码，多个码以逗号分割
    #[arg(long)]
    pub code: String,
    /// 猜码规则， Aa0
    #[arg(long)]
    pub rule: String,
    /// 猜码速度
    #[arg(long)]
    pub speed: Option<bool>,
}

#[derive(Parser)]
pub struct RedPacketArgs {
    /// 聊天ID，请使用 --chat-id='-123456789' 勿使用 --chat-id '-123456789'
    #[arg(short, long)]
    pub chat_id: i64,
    /// 是否归档中的聊天
    #[arg(short, long)]
    pub archive: bool,
    /// 点击按钮
    #[arg(long)]
    pub type_button: String,
}

#[derive(Parser)]
pub struct CreateAccountArgs {
    /// 聊天ID，请使用 --chat-id='-123456789' 勿使用 --chat-id '-123456789'
    #[arg(short, long)]
    pub chat_id: i64,
    /// 是否归档中的聊天
    #[arg(short, long)]
    pub archive: bool,
    /// 账户名
    #[arg(long)]
    pub account_name: String,
    /// 安全码
    #[arg(long)]
    pub security_code: String,
    /// 点击创建按钮的间隔时间
    #[arg(short, long)]
    pub type_button_interval_mills: u64,
}

#[derive(Parser)]
pub struct CreateAccountListenArgs {
    /// 聊天ID，请使用 --chat-id='-123456789' 勿使用 --chat-id '-123456789'
    #[arg(short, long)]
    pub chat_id: i64,
    /// bot聊天ID，请使用 --bot-chat-id='-123456789' 勿使用 --bot-chat-id '-123456789'
    #[arg(short, long)]
    pub bot_chat_id: i64,
    /// bot ID，请使用 --bot-id='-123456789' 勿使用 --bot-id '-123456789'
    #[arg(short, long)]
    pub bot_id: i64,
    /// 是否归档中的聊天
    #[arg(short, long)]
    pub archive: bool,
    /// bot chat是否归档中的聊天
    #[arg(short, long)]
    pub bot_archive: bool,
    /// 账户名
    #[arg(long)]
    pub account_name: String,
    /// 安全码
    #[arg(long)]
    pub security_code: String,
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: AppCommand,
}