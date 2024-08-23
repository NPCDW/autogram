use anyhow::anyhow;
use serde_json::json;
use tdlib_rs::{functions, enums};

use crate::{config::args_conf::FollowArgs, util::http_util};

use super::init_svc::{InitData, SimpleMessage};

pub async fn follow(init_data: InitData, follow_param: FollowArgs) -> anyhow::Result<()> {
    let client_id = init_data.client_id;
    if follow_param.forward_chat_id.is_some() {
        let forward_chat_id = follow_param.forward_chat_id.unwrap();
        // 需要先把聊天找到，才能监听聊天消息
        tracing::info!("查找转发目标聊天");
        let mut limit = 20;
        'find_chat: loop {
            tracing::debug!("查找聊天 limit: {}", limit);
            let chats = if follow_param.forward_chat_archive {
                functions::get_chats(Some(enums::ChatList::Archive), limit, client_id).await
            } else {
                functions::get_chats(None, limit, client_id).await
            };
            if chats.is_err() {
                return Err(anyhow!("获取聊天列表失败: {:?}", chats.as_ref().err()));
            }
            let enums::Chats::Chats(chats) = chats.unwrap();
            for chat_id in &chats.chat_ids {
                if chat_id == &forward_chat_id {
                    break 'find_chat;
                }
            }
            if chats.chat_ids.len() < limit as usize && limit > 20 {
                return Err(anyhow!("未找到ID为 {} 的聊天", forward_chat_id));
            }
            limit += 20;
        }
        tracing::info!("打开转发目标聊天");
        functions::open_chat(forward_chat_id, client_id).await.unwrap();
    }
    tracing::info!("监听消息");
    while let Some((new_msg, _)) = init_data.msg_rx.write().await.recv().await {
        if let Some(msg) = new_msg {
            let msg = msg.message;
            if follow_param.forward_chat_id.is_some() && msg.chat_id == follow_param.forward_chat_id.unwrap() {
                continue;
            }
            let sender_id = match msg.sender_id {
                enums::MessageSender::User(user) => user.user_id,
                enums::MessageSender::Chat(chat) => chat.chat_id,
            };
            if follow_param.user_id.contains(&sender_id) {
                tracing::info!("监听消息: {} {:?}", msg.id, msg.content);
                if follow_param.webhook_url.is_some() {
                    let msg = SimpleMessage {
                        id: msg.id,
                        chat_id: msg.chat_id,
                        content: msg.content.clone(),
                    };
                    let res = webhook(&msg, &follow_param.webhook_url.clone().unwrap()).await;
                    if res.is_err() {
                        return Err(anyhow!("webhook 消息失败 {:?}", res.err()));
                    }
                }
                if follow_param.forward_chat_id.is_some() {
                    let forward_chat_id = follow_param.forward_chat_id.unwrap();
                    // if msg.can_be_forwarded {
                    //     let message = functions::forward_messages(forward_chat_id, 0, msg.chat_id, vec![msg.id], None, false, false, client_id).await;
                    //     if message.is_err() {
                    //         return Err(anyhow!("发送消息失败: {:?}", message.as_ref().err()));
                    //     }
                    // }

                    if init_data.bot_token.is_none() {
                        tracing::warn!("未配置 BOT_TOKEN 环境变量，无法发送提醒消息");
                        continue;
                    }
                    let content = match &msg.content {
                        enums::MessageContent::MessageText(msg) => msg.text.text.clone(),
                        enums::MessageContent::MessagePhoto(msg) => msg.caption.text.clone(),
                        enums::MessageContent::MessageAudio(msg) => msg.caption.text.clone(),
                        enums::MessageContent::MessageDocument(msg) => msg.caption.text.clone(),
                        enums::MessageContent::MessageVideo(msg) => msg.caption.text.clone(),
                        _ => "暂不支持的消息类型".to_string(),
                    };
                    let link = match functions::get_message_link(msg.chat_id, msg.id, 0, false, false, client_id).await {
                        Ok(enums::MessageLink::MessageLink(link)) => link.link,
                        Err(_) => "nolink".to_string(),
                    };
                    let topic_id = if follow_param.forward_topic_id.is_none() { 0 } else { follow_param.forward_topic_id.unwrap() };
                    let url = format!("https://api.telegram.org/bot{}/sendMessage", init_data.bot_token.as_ref().unwrap());
                    let body = json!({"chat_id": forward_chat_id, "text": format!("{}\n{}", content, link), "message_thread_id": topic_id}).to_string();
                    tracing::debug!("forward 消息 body: {}", &body);
                    http_util::post(&url, body).await?;
                }
            }
        }
    }
    if follow_param.forward_chat_id.is_some() {
        let forward_chat_id = follow_param.forward_chat_id.unwrap();
        tracing::info!("关闭转发目标聊天");
        functions::close_chat(forward_chat_id, client_id).await.unwrap();
    }
    anyhow::Ok(())
}

async fn webhook(msg: &SimpleMessage, webhook_url: &String) -> anyhow::Result<()> {
    let content = match &msg.content {
        enums::MessageContent::MessageText(msg) => msg.text.text.clone(),
        enums::MessageContent::MessagePhoto(msg) => msg.caption.text.clone(),
        enums::MessageContent::MessageAudio(msg) => msg.caption.text.clone(),
        enums::MessageContent::MessageDocument(msg) => msg.caption.text.clone(),
        enums::MessageContent::MessageVideo(msg) => msg.caption.text.clone(),
        _ => return anyhow::Ok(()),
    };
    let json = json!({"id": msg.id, "content": content}).to_string();
    tracing::debug!("webhook 消息 json: {}", &json);
    http_util::post(webhook_url, json).await?;
    anyhow::Ok(())
}