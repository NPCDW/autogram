use anyhow::anyhow;
use serde_json::json;
use tdlib_rs::{functions, enums, types};

use crate::{config::args_conf::FollowArgs, util::http_util};

use super::init_svc::{InitData, SimpleMessage};

pub async fn follow(init_data: InitData, follow_param: FollowArgs) -> anyhow::Result<()> {
    let client_id = init_data.client_id;
    if follow_param.forward_chat_id.is_some() {
        let forward_chat_id = follow_param.forward_chat_id.unwrap();
        // 需要先把聊天找到，才能监听聊天消息
        let mut limit = 20;
        'find_chat: loop {
            tracing::debug!("查找聊天 limit: {}", limit);
            let chats = functions::get_chats(None, limit, client_id).await;
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
        tracing::debug!("打开聊天");
        functions::open_chat(forward_chat_id, client_id).await.unwrap();
    }
    tracing::debug!("监听消息");
    while let Some((new_msg, _)) = init_data.msg_rx.write().await.recv().await {
        if let Some(msg) = new_msg {
            let msg = msg.message;
            if let enums::MessageSender::User(user) = msg.sender_id {
                if follow_param.forward_chat_id.is_some() && msg.chat_id == follow_param.forward_chat_id.unwrap() {
                    continue;
                }
                if follow_param.user_id.contains(&user.user_id) {
                    tracing::info!("监听消息: {} {:?}", msg.id, msg.content);
                    if follow_param.webhook_url.is_some() {
                        let msg = SimpleMessage {
                            id: msg.id,
                            chat_id: msg.chat_id,
                            content: msg.content,
                        };
                        let res = webhook(&msg, &follow_param.webhook_url.clone().unwrap()).await;
                        if res.is_err() {
                            return Err(anyhow!("webhook 消息失败 {:?}", res.err()));
                        }
                    }
                    if follow_param.forward_chat_id.is_some() {
                        let forward_chat_id = follow_param.forward_chat_id.unwrap();
                        let reply_to = if msg.can_be_forwarded {
                            Some(enums::InputMessageReplyTo::Message(types::InputMessageReplyToMessage {
                                chat_id: msg.chat_id,
                                message_id: msg.id,
                                quote: None,
                            }))
                        } else {
                            None
                        };
                        let res = functions::get_message_link(msg.chat_id, msg.id, 0, false, false, client_id).await;
                        if res.is_err() {
                            return Err(anyhow!("获取消息链接失败 {:?}", res.err()));
                        }
                        let enums::MessageLink::MessageLink(link) = res.unwrap();
                        let input_message_content = enums::InputMessageContent::InputMessageText(types::InputMessageText {
                            text: types::FormattedText {
                                text: link.link,
                                entities: vec![]
                            },
                            link_preview_options: None,
                            clear_draft: true
                        });
                        let message = functions::send_message(forward_chat_id, 0, reply_to, None, input_message_content, client_id).await;
                        if message.is_err() {
                            return Err(anyhow!("发送消息失败: {:?}", message.as_ref().err()));
                        }
                    }
                }
            }
        }
    }
    if follow_param.forward_chat_id.is_some() {
        let forward_chat_id = follow_param.forward_chat_id.unwrap();
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