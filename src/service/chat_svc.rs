use std::time::Duration;

use anyhow::anyhow;
use tdlib_rs::{functions, enums, types};

use crate::config::args_conf::ChatArgs;

use super::init_svc::InitData;

pub async fn chat(init_data: InitData, chat_param: ChatArgs) -> anyhow::Result<()> {
    let client_id = init_data.client_id;
    // 需要先把聊天找到，才能向聊天发送消息
    tracing::info!("查找聊天");
    let mut limit = 20;
    'find_chat: loop {
        tracing::debug!("查找聊天 limit: {}", limit);
        let chats = if chat_param.archive {
            functions::get_chats(Some(enums::ChatList::Archive), limit, client_id).await
        } else {
            functions::get_chats(None, limit, client_id).await
        };
        if chats.is_err() {
            return Err(anyhow!("获取聊天列表失败: {:?}", chats.as_ref().err()));
        }
        let enums::Chats::Chats(chats) = chats.unwrap();
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
    tracing::info!("打开聊天");
    functions::open_chat(chat_param.chat_id, client_id).await.unwrap();
    tracing::info!("发送消息");
    let message = functions::send_message(chat_param.chat_id, 0, None, None, 
        enums::InputMessageContent::InputMessageText(types::InputMessageText {
            text: types::FormattedText {
                text: chat_param.message,
                entities: vec![]
            },
            link_preview_options: None,
            clear_draft: true
        }), client_id).await;
    if message.is_err() {
        return Err(anyhow!("发送消息失败: {:?}", message.as_ref().err()));
    }
    let enums::Message::Message(message) = message.unwrap();
    tracing::info!("发送消息中 id: {} content: {:?}", message.id, message.content);
    // 等待消息发送完成
    let timeout = tokio::time::timeout(tokio::time::Duration::from_secs(5), async {
        while let Some((new_msg, _)) = init_data.msg_rx.write().await.recv().await {
            if let Some(msg) = new_msg {
                if msg.message.id == message.id {
                    tracing::info!("消息 {} 发送成功", message.id);
                    // 如果不在此处睡眠1秒，消息有些时候会发送失败，原因不明
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    break;
                }
            }
        }
    }).await;
    if let Some(type_button) = chat_param.type_button {
        let _ = tokio::time::timeout(Duration::from_secs(5), async {
            while let Some((new_msg, update_msg)) = init_data.msg_rx.write().await.recv().await {
                if let Some(new_msg) = new_msg {
                    if new_msg.message.chat_id == chat_param.chat_id {
                        tracing::info!("监听消息: {} {:?} {:?}", new_msg.message.id, new_msg.message.content, new_msg.message.reply_markup);
                        if let Some(reply) = new_msg.message.reply_markup {
                            if let enums::ReplyMarkup::InlineKeyboard(reply) = reply {
                                for row in reply.rows {
                                    for button in row {
                                        tracing::debug!("按钮: {:?}", button);
                                        if button.text == type_button {
                                            if let enums::InlineKeyboardButtonType::Callback(button_type) = button.r#type {
                                                let payload = enums::CallbackQueryPayload::Data(types::CallbackQueryPayloadData { data: button_type.data });
                                                let res = functions::get_callback_query_answer(chat_param.chat_id, new_msg.message.id, payload, client_id).await;
                                                if let Ok(enums::CallbackQueryAnswer::CallbackQueryAnswer(answer)) = res {
                                                    tracing::info!("内嵌键盘点击成功: {:?}", answer);
                                                } else {
                                                    tracing::error!("内嵌键盘点击 error: {:?}", res);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            tracing::error!("发送消息后，收到的消息没有任何按钮");
                        }
                    }
                } else if let Some(update_msg) = update_msg {
                    if update_msg.chat_id == chat_param.chat_id {
                        tracing::info!("监听到消息内容变更: {} {:?}", update_msg.message_id, update_msg.new_content);
                    }
                }
            }
        }).await;
    }
    tracing::info!("关闭聊天");
    functions::close_chat(chat_param.chat_id, client_id).await.unwrap();
    if timeout.is_err() {
        return Err(anyhow!("发送消息失败: {:?}", timeout.err()));
    }
    anyhow::Ok(())
}