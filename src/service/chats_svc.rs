use anyhow::anyhow;
use tdlib_rs::{functions, enums};

pub async fn top(client_id: i32, limit: i32) -> anyhow::Result<()> {
    // Run the get_me() method to get user information
    let chats = functions::get_chats(None, limit, client_id).await;
    if chats.is_err() {
        return Err(anyhow!("获取前二十个聊天列表失败: {:?}", chats.as_ref().err()));
    }
    let enums::Chats::Chats(chats) = chats.unwrap();
    for chat_id in chats.chat_ids {
        let chat = functions::get_chat(chat_id, client_id).await;
        if chat.is_err() {
            tracing::error!("获取 id 为 {} 的聊天失败: {:?}", chat_id, chat.as_ref().err());
            continue;
        }
        let enums::Chat::Chat(chat) = chat.unwrap();
        tracing::info!("chat_id: {} title: {}", chat.id, chat.title);
        if chat.view_as_topics {
            let topics = functions::get_forum_topics(chat.id, "".to_string(), 0, 0, 0, 100, client_id).await;
            if topics.is_err() {
                tracing::error!("  └─ 获取该聊天下的主题失败：{:?}", topics.err());
            } else {
                let enums::ForumTopics::ForumTopics(topics) = topics.unwrap();
                for topic in topics.topics {
                    tracing::info!("  └─ topic_id: {} title: {}", topic.info.message_thread_id, topic.info.name);
                }
            }
        }
    }
    anyhow::Ok(())
}