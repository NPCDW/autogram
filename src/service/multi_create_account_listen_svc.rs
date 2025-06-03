use std::collections::HashMap;

use anyhow::anyhow;
use tdlib_rs::{enums, functions};

use crate::config::args_conf::{CreateAccountArgs, CreateAccountListenArgs, MultiCreateAccountListenArgs};

use super::init_svc::InitData;

pub async fn create(init_data: InitData, param: MultiCreateAccountListenArgs) -> anyhow::Result<()> {
    let mut args = HashMap::new();
    for item in param.multi {
        let item = serde_json::from_str::<CreateAccountListenArgs>(&item);
        match item {
            Err(e) => return Err(anyhow!("解析参数失败: {:?}", e)),
            Ok(v) => {args.insert(v.chat_id, v);},
        }
    }
    let client_id = init_data.client_id;
    // 需要先把聊天找到，才能监听聊天消息
    tracing::info!("查找聊天");
    for arg in args.values().collect::<Vec<&CreateAccountListenArgs>>() {
        let mut limit = 20;
        'find_chat: loop {
            tracing::debug!("查找聊天 limit: {}", limit);
            let chats = if arg.archive {
                functions::get_chats(Some(enums::ChatList::Archive), limit, client_id).await
            } else {
                functions::get_chats(None, limit, client_id).await
            };
            if chats.is_err() {
                return Err(anyhow!("获取聊天列表失败: {:?}", chats.as_ref().err()));
            }
            let enums::Chats::Chats(chats) = chats.unwrap();
            for chat_id in &chats.chat_ids {
                if chat_id == &arg.chat_id {
                    break 'find_chat;
                }
            }
            if chats.chat_ids.len() < limit as usize && limit > 20 {
                return Err(anyhow!("未找到ID为 {} 的聊天", arg.chat_id));
            }
            limit += 20;
        }
        tracing::info!("打开聊天");
        functions::open_chat(arg.chat_id, client_id).await.unwrap();
    }
    tracing::info!("监听自由注册消息开始");
    let chat_ids = args.keys().collect::<Vec<&i64>>();
    'receiving_messages: while let Some((new_msg, _update_msg)) = init_data.msg_rx.write().await.recv().await {
        if let Some(new_msg) = new_msg {
            let sender_id = match new_msg.message.sender_id {
                enums::MessageSender::User(user) => user.user_id,
                enums::MessageSender::Chat(chat) => chat.chat_id,
            };
            if chat_ids.contains(&&new_msg.message.chat_id) && sender_id == args[&new_msg.message.chat_id].bot_id {
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
                    let init_data_clone = init_data.clone();
                    let arg_clone = args[&new_msg.message.chat_id].clone();
                    tokio::spawn(async move {
                        let res = super::create_account_svc::create(init_data_clone, CreateAccountArgs {
                            chat_id: arg_clone.bot_id,
                            archive: arg_clone.bot_archive,
                            account_name: arg_clone.account_name.clone(),
                            security_code: arg_clone.security_code.clone(),
                            type_button_interval_mills: 100,
                        }).await;
                        if let Err(e) = res {
                            tracing::error!("创建账号失败: {:?}", e)
                        }
                    });
                    continue 'receiving_messages;
                }
                if let Some(code_prefix) = args[&new_msg.message.chat_id].code_prefix.clone() {
                    let lines = content.split("\n").collect::<Vec<&str>>();
                    for line in lines {
                        if line.starts_with(&code_prefix) {
                            let res = crate::service::guess_code_svc::use_code(&init_data, line, args[&new_msg.message.chat_id].bot_id, Some(args[&new_msg.message.chat_id].bot_archive), client_id).await;
                            if let Err(err) = res {
                                tracing::error!("使用注册码失败: {}", err);
                            } else {
                                tracing::info!("使用注册码成功 {}", line);
                                return anyhow::Ok(());
                            }
                        }
                    }
                }
            }
        }
    }
    tracing::info!("关闭聊天");
    for chat_id in chat_ids {
        functions::close_chat(chat_id.to_owned(), client_id).await.unwrap();
    }
    anyhow::Ok(())
}