use anyhow::anyhow;
use tdlib_rs::{enums, functions, types};

use crate::config::args_conf::RedPacketArgs;

use super::init_svc::InitData;

pub async fn grab(init_data: InitData, listen_param: RedPacketArgs) -> anyhow::Result<()> {
    let client_id = init_data.client_id;
    // 需要先把聊天找到，才能监听聊天消息
    tracing::info!("查找聊天");
    let mut limit = 20;
    'find_chat: loop {
        tracing::debug!("查找聊天 limit: {}", limit);
        let chats = if listen_param.archive {
            functions::get_chats(Some(enums::ChatList::Archive), limit, client_id).await
        } else {
            functions::get_chats(None, limit, client_id).await
        };
        if chats.is_err() {
            return Err(anyhow!("获取聊天列表失败: {:?}", chats.as_ref().err()));
        }
        let enums::Chats::Chats(chats) = chats.unwrap();
        for chat_id in &chats.chat_ids {
            if chat_id == &listen_param.chat_id {
                break 'find_chat;
            }
        }
        if chats.chat_ids.len() < limit as usize && limit > 20 {
            return Err(anyhow!("未找到ID为 {} 的聊天", listen_param.chat_id));
        }
        limit += 20;
    }
    tracing::info!("打开聊天");
    functions::open_chat(listen_param.chat_id, client_id).await.unwrap();
    tracing::info!("监听消息");
    while let Some((new_msg, _new_content)) = init_data.msg_rx.write().await.recv().await {
        if let Some(new_msg) = new_msg {
            if new_msg.message.chat_id == listen_param.chat_id {
                if let Some(reply) = new_msg.message.reply_markup {
                    if let enums::ReplyMarkup::InlineKeyboard(reply) = reply {
                        tracing::info!("监听到带按钮消息: {} {:?}", new_msg.message.id, new_msg.message.content);
                        for row in reply.rows {
                            for button in row {
                                tracing::debug!("按钮: {:?}", button);
                                if button.text == listen_param.type_button {
                                    if let enums::InlineKeyboardButtonType::Callback(button_type) = button.r#type {
                                        let payload = enums::CallbackQueryPayload::Data(types::CallbackQueryPayloadData { data: button_type.data });
                                        let res = functions::get_callback_query_answer(listen_param.chat_id, new_msg.message.id, payload, client_id).await;
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
                    tracing::error!("最新收到的消息没有任何按钮");
                }
            }
        }
    }
    tracing::info!("关闭聊天");
    functions::close_chat(listen_param.chat_id, client_id).await.unwrap();
    anyhow::Ok(())
}