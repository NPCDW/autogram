use anyhow::anyhow;
use tdlib_rs::{enums, functions, types};

use crate::config::args_conf::CreateAccountArgs;

use super::init_svc::InitData;

pub async fn create(init_data: InitData, param: CreateAccountArgs) -> anyhow::Result<()> {
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
    tracing::info!("发送消息");
    let message = functions::send_message(param.chat_id, 0, None, None, 
        enums::InputMessageContent::InputMessageText(types::InputMessageText {
            text: types::FormattedText {
                text: "/start".to_string(),
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
    // 点击回复消息中的按钮
    'receiving_messages: while let Some((new_msg, _update_msg)) = init_data.msg_rx.write().await.recv().await {
        if let Some(new_msg) = new_msg {
            if new_msg.message.chat_id == param.chat_id {
                tracing::info!("监听消息: {} {:?} {:?}", new_msg.message.id, new_msg.message.content, new_msg.message.reply_markup);
                if let Some(reply) = new_msg.message.reply_markup {
                    if let enums::ReplyMarkup::InlineKeyboard(reply) = reply {
                        for row in reply.rows {
                            for button in row {
                                tracing::debug!("按钮: {:?}", button);
                                if button.text.contains("创建账户") {
                                    if let enums::InlineKeyboardButtonType::Callback(button_type) = button.r#type {
                                        loop {
                                            let payload = enums::CallbackQueryPayload::Data(types::CallbackQueryPayloadData { data: button_type.data.clone() });
                                            let res = functions::get_callback_query_answer(param.chat_id, new_msg.message.id, payload, client_id).await;
                                            if let Ok(enums::CallbackQueryAnswer::CallbackQueryAnswer(answer)) = res {
                                                tracing::info!("内嵌键盘点击成功: {:?}", answer);
                                                if answer.text.contains("请勿重复注册") {
                                                    break 'receiving_messages;
                                                }
                                                if answer.text.contains("自助注册已关闭") {
                                                    tokio::time::sleep(tokio::time::Duration::from_millis(param.type_button_interval_mills)).await;
                                                    continue;
                                                }
                                                break;
                                            } else {
                                                tracing::error!("内嵌键盘点击 error: {:?}", res);
                                                tokio::time::sleep(tokio::time::Duration::from_millis(param.type_button_interval_mills)).await;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else if let enums::MessageContent::MessageText(text) = new_msg.message.content {
                    if text.text.text.contains("您已进入注册状态") {
                        tracing::info!("发送账户信息消息");
                        let message = functions::send_message(param.chat_id, 0, None, None, 
                            enums::InputMessageContent::InputMessageText(types::InputMessageText {
                                text: types::FormattedText {
                                    text: format!("{} {}", param.account_name, param.security_code),
                                    entities: vec![]
                                },
                                link_preview_options: None,
                                clear_draft: true
                            }), client_id).await;
                        if message.is_err() {
                            return Err(anyhow!("发送账户信息消息失败: {:?}", message.as_ref().err()));
                        }
                    }
                } else {
                    tracing::error!("最新收到的消息没有任何按钮");
                }
            }
        }
    }
    tracing::info!("关闭聊天");
    functions::close_chat(param.chat_id, client_id).await.unwrap();
    anyhow::Ok(())
}