use anyhow::anyhow;
use tdlib::functions;

pub async fn checkin(client_id: i32) -> anyhow::Result<()> {
    let akile_chat_id = std::env::var("AKILE_CHAT_ID");
    if akile_chat_id.is_err() {
        return Err(anyhow!("AKILE_CHAT_ID 环境变量配置错误，跳过 akile 签到， AKILE_CHAT_ID: {:?}", akile_chat_id));
    }
    let akile_chat_id = akile_chat_id.unwrap().parse().unwrap();
    // 需要先把聊天找到，才能向聊天发送消息
    let mut limit = 20;
    'find_chat: loop {
        let chats = functions::get_chats(None, limit, client_id).await;
        if chats.is_err() {
            return Err(anyhow!("获取聊天列表失败: {:?}", chats.as_ref().err()));
        }
        let tdlib::enums::Chats::Chats(chats) = chats.unwrap();
        if chats.chat_ids.len() < limit as usize {
            return Err(anyhow!("未找到ID为 {} 的聊天", akile_chat_id));
        }
        for chat_id in chats.chat_ids {
            if chat_id == akile_chat_id {
                break 'find_chat;
            }
        }
        limit += 20;
    }
    functions::open_chat(akile_chat_id, client_id).await.unwrap();
    let message = functions::send_message(akile_chat_id, 0, None, None, None,
        tdlib::enums::InputMessageContent::InputMessageText(tdlib::types::InputMessageText {
            text: tdlib::types::FormattedText {
                text: "/checkin@akilecloud_bot".to_string(),
                entities: vec![]
            },
            disable_web_page_preview: true,
            clear_draft: true
        }), client_id).await;
    if message.is_err() {
        return Err(anyhow!("发送 akile 签到失败: {:?}", message.as_ref().err()));
    }
    let tdlib::enums::Message::Message(message) = message.unwrap();
    tracing::info!("发送 akile 签到消息 id: {} content: {:?}", message.id, message.content);
    functions::close_chat(akile_chat_id, client_id).await.unwrap();
    anyhow::Ok(())
}