use anyhow::anyhow;
use tdlib::functions;

pub async fn top(client_id: i32, limit: i32) -> anyhow::Result<()> {
    // Run the get_me() method to get user information
    let chats = functions::get_chats(None, limit, client_id).await;
    if chats.is_err() {
        return Err(anyhow!("获取前二十个聊天列表失败: {:?}", chats.as_ref().err()));
    }
    let tdlib::enums::Chats::Chats(chats) = chats.unwrap();
    for chat_id in chats.chat_ids {
        let chat = functions::get_chat(chat_id, client_id).await;
        if chat.is_err() {
            tracing::error!("获取 id 为 {} 的聊天失败: {:?}", chat_id, chat.as_ref().err());
            continue;
        }
        let tdlib::enums::Chat::Chat(chat) = chat.unwrap();
        tracing::info!("id: {} title: {}", chat.id, chat.title);
    }
    anyhow::Ok(())
}