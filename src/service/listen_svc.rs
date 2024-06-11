use anyhow::anyhow;
use tdlib::functions;

use crate::config::args_conf::ListenArgs;

use super::init_svc::InitData;

pub async fn listen(init_data: InitData, listen_param: ListenArgs) -> anyhow::Result<()> {
    let client_id = init_data.client_id;
    // 需要先把聊天找到，才能向聊天发送消息
    tracing::debug!("查找聊天");
    let mut limit = 20;
    'find_chat: loop {
        let chats = functions::get_chats(None, limit, client_id).await;
        if chats.is_err() {
            return Err(anyhow!("获取聊天列表失败: {:?}", chats.as_ref().err()));
        }
        let tdlib::enums::Chats::Chats(chats) = chats.unwrap();
        if chats.chat_ids.len() < limit as usize && limit > 20 {
            return Err(anyhow!("未找到ID为 {} 的聊天", listen_param.chat_id));
        }
        for chat_id in chats.chat_ids {
            if chat_id == listen_param.chat_id {
                break 'find_chat;
            }
        }
        limit += 20;
    }
    tracing::debug!("打开聊天");
    functions::open_chat(listen_param.chat_id, client_id).await.unwrap();
    tracing::debug!("查询历史消息");
    if listen_param.history {
        let mut from_message_id = 0;
        let limit = 10;
        let mut total = 0;
        loop {
            let history = functions::get_chat_history(listen_param.chat_id, from_message_id, 0, limit, false, client_id).await;
            if history.is_err() {
                return Err(anyhow!("获取历史消息失败 {:?}", history.err()));
            }
            let tdlib::enums::Messages::Messages(messages) = history.unwrap();
            total += messages.messages.len();
            if messages.messages.len() <= 0 || total >= listen_param.max_history {
                tracing::info!("历史消息获取完成 {:?}", messages);
                break;
            }
            tracing::info!("历史消息: {} {:?}", messages.total_count, messages.messages);
            from_message_id = messages.messages[messages.messages.len() - 1].as_ref().unwrap().id;
        }
    }
    tracing::debug!("监听消息");
    while let Some(msg) = init_data.msg_rx.write().await.recv().await {
        if msg.chat_id == listen_param.chat_id {
            tracing::info!("监听消息: {} {:?}", msg.id, msg.content);
        }
    }
    functions::close_chat(listen_param.chat_id, client_id).await.unwrap();
    anyhow::Ok(())
}