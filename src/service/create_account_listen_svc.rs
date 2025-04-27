use anyhow::anyhow;
use tdlib_rs::{enums, functions};

use crate::config::args_conf::{CreateAccountArgs, CreateAccountListenArgs};

use super::init_svc::InitData;

pub async fn create(init_data: InitData, param: CreateAccountListenArgs) -> anyhow::Result<()> {
    let client_id = init_data.client_id;
    // 需要先把聊天找到，才能监听聊天消息
    tracing::info!("查找聊天");
    let mut limit = 20;
    'find_chat: loop {
        tracing::debug!("查找聊天 limit: {}", limit);
        let chats = if param.archive {
            functions::get_chats(Some(enums::ChatList::Archive), limit, client_id).await
        } else {
            functions::get_chats(None, limit, client_id).await
        };
        if chats.is_err() {
            return Err(anyhow!("获取聊天列表失败: {:?}", chats.as_ref().err()));
        }
        let enums::Chats::Chats(chats) = chats.unwrap();
        for chat_id in &chats.chat_ids {
            if chat_id == &param.chat_id {
                break 'find_chat;
            }
        }
        if chats.chat_ids.len() < limit as usize && limit > 20 {
            return Err(anyhow!("未找到ID为 {} 的聊天", param.chat_id));
        }
        limit += 20;
    }
    tracing::info!("打开聊天");
    functions::open_chat(param.chat_id, client_id).await.unwrap();
    tracing::info!("监听自由注册消息开始");
    'receiving_messages: while let Some((new_msg, _update_msg)) = init_data.msg_rx.write().await.recv().await {
        if let Some(new_msg) = new_msg {
            let sender_id = match new_msg.message.sender_id {
                enums::MessageSender::User(user) => user.user_id,
                enums::MessageSender::Chat(chat) => chat.chat_id,
            };
            if new_msg.message.chat_id == param.chat_id && sender_id == param.bot_id {
                tracing::info!("监听消息: {} {:?} {:?}", new_msg.message.id, new_msg.message.content, new_msg.message.reply_markup);
                let content = match &new_msg.message.content {
                    enums::MessageContent::MessageText(msg) => msg.text.text.clone(),
                    enums::MessageContent::MessagePhoto(msg) => msg.caption.text.clone(),
                    enums::MessageContent::MessageAudio(msg) => msg.caption.text.clone(),
                    enums::MessageContent::MessageDocument(msg) => msg.caption.text.clone(),
                    enums::MessageContent::MessageVideo(msg) => msg.caption.text.clone(),
                    _ => "".to_string(),
                };
                if (content.contains("自由注册") || content.contains("定时注册")) && content.contains("已开启") {
                    break 'receiving_messages;
                }
            }
        }
    }
    super::create_account_svc::create(init_data, CreateAccountArgs {
        chat_id: param.bot_chat_id,
        archive: param.bot_archive,
        account_name: param.account_name.clone(),
        security_code: param.security_code.clone(),
        type_button_interval_mills: 100,
    }).await?;
    tracing::info!("关闭聊天");
    functions::close_chat(param.chat_id, client_id).await.unwrap();
    anyhow::Ok(())
}