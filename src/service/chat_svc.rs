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
        let chats = if let Some(true) = chat_param.archive {
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
    // 点击回复消息中的按钮
    if let Some(type_button) = chat_param.type_button {
        let mut type_index = 0;
        let _ = tokio::time::timeout(Duration::from_secs(chat_param.timeout.unwrap_or(5)), async {
            while let Some((new_msg, update_msg)) = init_data.msg_rx.write().await.recv().await {
                if let Some(new_msg) = new_msg {
                    if new_msg.message.chat_id == chat_param.chat_id {
                        tracing::info!("监听消息: {} {:?} {:?}", new_msg.message.id, new_msg.message.content, new_msg.message.reply_markup);
                        if let Some(reply) = new_msg.message.reply_markup {
                            let type_res = tokio::time::timeout(Duration::from_secs(chat_param.single_step_timeout.unwrap_or(5)),
                            type_reply_button(reply, chat_param.chat_id, new_msg.message.id, client_id, type_index, &type_button)).await;
                            match type_res {
                                Ok(Ok(false)) => (),
                                Ok(Err(e)) => tracing::error!("点击按钮失败: {:?}", e),
                                _ => {
                                    type_index += 1;
                                    if type_index >= type_button.len() {
                                        break;
                                    }
                                },
                            };
                        }
                    }
                } else if let Some(update_msg) = update_msg {
                    if update_msg.chat_id == chat_param.chat_id {
                        tracing::info!("监听到消息内容变更: {} {:?}", update_msg.message_id, update_msg.new_content);
                        let res = functions::get_message(chat_param.chat_id, update_msg.message_id, client_id).await;
                        if let Ok(enums::Message::Message(message)) = res {
                            tracing::info!("获取消息内容变更: {} {:?} {:?}", message.id, message.content, message.reply_markup);
                            if let Some(reply) = message.reply_markup {
                                let type_res = tokio::time::timeout(Duration::from_secs(chat_param.single_step_timeout.unwrap_or(5)),
                                type_reply_button(reply, chat_param.chat_id, message.id, client_id, type_index, &type_button)).await;
                                match type_res {
                                    Ok(Ok(false)) => (),
                                    Ok(Err(e)) => tracing::error!("点击按钮失败: {:?}", e),
                                    _ => {
                                        type_index += 1;
                                        if type_index >= type_button.len() {
                                            break;
                                        }
                                    },
                                };
                            }
                        } else {
                            tracing::error!("获取消息内容变更失败: {:?}", res);
                        }
                    }
                }
            }
        }).await;
    }
    // 转发回复的消息
    if let Some(forward_chat_id) = chat_param.forward_chat_id {
        let _ = tokio::time::timeout(Duration::from_secs(5), async {
            while let Some((new_msg, update_msg)) = init_data.msg_rx.write().await.recv().await {
                if let Some(new_msg) = new_msg {
                    if new_msg.message.chat_id == chat_param.chat_id {
                        tracing::info!("监听消息: {} {:?} {:?}", new_msg.message.id, new_msg.message.content, new_msg.message.reply_to);
                        let forward_messages = functions::forward_messages(forward_chat_id, chat_param.forward_topic_id.unwrap_or(0), chat_param.chat_id, vec![new_msg.message.id], None, false, false, client_id).await;
                        if forward_messages.is_err() {
                            tracing::error!("转发消息失败: {:?}", forward_messages);
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

async fn type_reply_button(reply: enums::ReplyMarkup, chat_id: i64, message_id: i64, client_id: i32, type_index: usize, type_button: &Vec<String>) -> anyhow::Result<bool> {
    if let enums::ReplyMarkup::InlineKeyboard(reply) = reply {
        for row in reply.rows {
            for button in row {
                tracing::debug!("按钮: {:?}", button);
                if button.text == type_button[type_index] {
                    tokio::time::sleep(tokio::time::Duration::from_millis(700)).await;
                    if let enums::InlineKeyboardButtonType::Callback(button_type) = button.r#type {
                        let payload = enums::CallbackQueryPayload::Data(types::CallbackQueryPayloadData { data: button_type.data });
                        let res = functions::get_callback_query_answer(chat_id, message_id, payload, client_id).await;
                        if let Ok(enums::CallbackQueryAnswer::CallbackQueryAnswer(answer)) = res {
                            tracing::info!("内嵌键盘点击成功: {:?}", answer);
                            return anyhow::Ok(true);
                        } else {
                            return Err(anyhow!("内嵌键盘点击 error: {:?}", res));
                        }
                    } else if let enums::InlineKeyboardButtonType::WebApp(button_type) = button.r#type {
                        let res = functions::open_web_app(chat_id, chat_id, button_type.url, None, "应用名称".to_string(), 0, None, client_id).await;
                        if let Ok(enums::WebAppInfo::WebAppInfo(web_app_info)) = res {
                            tracing::info!("内嵌键盘点击成功: {:?}", web_app_info);
                        } else {
                            return Err(anyhow!("内嵌键盘点击 error: {:?}", res));
                        }
                    }
                }
            }
        }
    }
    anyhow::Ok(false)
}