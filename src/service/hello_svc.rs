use anyhow::anyhow;
use tdlib::{enums::User, functions};

pub async fn hello(client_id: i32) -> anyhow::Result<()> {
    // Run the get_me() method to get user information
    let me = functions::get_me(client_id).await;
    if me.is_err() {
        tracing::error!("请先登录");
        functions::close(client_id).await.unwrap();
        return Err(anyhow!("用户未登录"));
    }
    let User::User(me) = functions::get_me(client_id).await.unwrap();
    tracing::info!("Hello {} Welcome to use autogram", me.first_name);

    let chats = functions::get_chats(None, 20, client_id).await;
    if chats.is_err() {
        tracing::error!("获取前二十个聊天列表失败: {:?}", chats.as_ref().err());
        return Err(anyhow!("获取前二十个聊天列表失败"));
    }
    let tdlib::enums::Chats::Chats(chats) = chats.unwrap();
    for chat_id in chats.chat_ids {
        let chat = functions::get_chat(chat_id, client_id).await;
        if chat.is_err() {
            tracing::error!("获取 id 为 {} 的聊天失败: {:?}", chat_id, chat.as_ref().err());
            return Err(anyhow!("获取某个聊天列表失败"));
        }
        let tdlib::enums::Chat::Chat(chat) = chat.unwrap();
        tracing::info!("title: {} id: {}", chat.title, chat.id);
    }
    anyhow::Ok(())
}