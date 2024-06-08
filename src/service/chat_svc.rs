use anyhow::anyhow;
use tdlib::functions;

use crate::config::args_conf::ChatArgs;

pub async fn chat(client_id: i32, chat_param: ChatArgs) -> anyhow::Result<()> {
    // 需要先把聊天找到，才能向聊天发送消息
    let mut limit = 20;
    'find_chat: loop {
        let chats = functions::get_chats(None, limit, client_id).await;
        if chats.is_err() {
            return Err(anyhow!("获取聊天列表失败: {:?}", chats.as_ref().err()));
        }
        let tdlib::enums::Chats::Chats(chats) = chats.unwrap();
        if chats.chat_ids.len() < limit as usize && limit > 20 {
            return Err(anyhow!("未找到ID为 {} 的聊天", chat_param.chat_id));
        }
        for chat_id in chats.chat_ids {
            if chat_id == chat_param.chat_id {
                break 'find_chat;
            }
        }
        limit += 20;
    }
    functions::open_chat(chat_param.chat_id, client_id).await.unwrap();
    let message = functions::send_message(chat_param.chat_id, 0, None, None, None,
        tdlib::enums::InputMessageContent::InputMessageText(tdlib::types::InputMessageText {
            text: tdlib::types::FormattedText {
                text: chat_param.message,
                entities: vec![]
            },
            disable_web_page_preview: true,
            clear_draft: true
        }), client_id).await;
    if message.is_err() {
        return Err(anyhow!("发送消息失败: {:?}", message.as_ref().err()));
    }
    let tdlib::enums::Message::Message(message) = message.unwrap();
    tracing::info!("发送消息 id: {} content: {:?}", message.id, message.content);
    // 等待消息发送完成
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    functions::close_chat(chat_param.chat_id, client_id).await.unwrap();
    anyhow::Ok(())
}