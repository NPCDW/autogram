use anyhow::anyhow;
use tdlib_rs::{functions, enums, types};

use crate::config::args_conf::GuessCodeArgs;

use super::init_svc::InitData;

pub async fn guess_code(init_data: InitData, guess_code_param: GuessCodeArgs) -> anyhow::Result<()> {
    let client_id = init_data.client_id;
    // 需要先把聊天找到，才能向聊天发送消息
    tracing::info!("查找聊天");
    let mut limit = 20;
    'find_chat: loop {
        tracing::debug!("查找聊天 limit: {}", limit);
        let chats = if let Some(true) = guess_code_param.archive {
            functions::get_chats(Some(enums::ChatList::Archive), limit, client_id).await
        } else {
            functions::get_chats(None, limit, client_id).await
        };
        if chats.is_err() {
            return Err(anyhow!("获取聊天列表失败: {:?}", chats.as_ref().err()));
        }
        let enums::Chats::Chats(chats) = chats.unwrap();
        if chats.chat_ids.len() < limit as usize && limit > 20 {
            return Err(anyhow!("未找到ID为 {} 的聊天", guess_code_param.chat_id));
        }
        for chat_id in chats.chat_ids {
            if chat_id == guess_code_param.chat_id {
                break 'find_chat;
            }
        }
        limit += 20;
    }
    tracing::info!("打开聊天");
    functions::open_chat(guess_code_param.chat_id, client_id).await.unwrap();

    if !guess_code_param.code.contains("*") {
        tracing::info!("{} 不包含 * 字符，猜码格式不正确", guess_code_param.code);
        return anyhow::Ok(());
    }
    let blank = guess_code_param.code.replace("*", "");
    if blank.len() + 1 < guess_code_param.code.len() {
        return Err(anyhow::anyhow!("出现了太多的 * 请确保只有一个"));
    }
    let mut code_final_list = vec![];
    if guess_code_param.rule.contains("A") {
        for c in 'A'..='Z' {
            code_final_list.push(guess_code_param.code.replace("*", &c.to_string()));
        }
    }
    if guess_code_param.rule.contains("a") {
        for c in 'a'..='z' {
            code_final_list.push(guess_code_param.code.replace("*", &c.to_string()));
        }
    }
    if guess_code_param.rule.contains("0") {
        for c in 0..=9 {
            code_final_list.push(guess_code_param.code.replace("*", &c.to_string()));
        }
    }
    // 将一个码所有的可能全部发送
    for code in code_final_list {
        tracing::info!("发送消息");
        let message = functions::send_message(guess_code_param.chat_id, 0, None, None, 
            enums::InputMessageContent::InputMessageText(types::InputMessageText {
                text: types::FormattedText {
                    text: format!("/start {}", code),
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
        if let Some(true) = guess_code_param.fast {} else {
            let status = wait_code_status(&init_data, guess_code_param.chat_id).await;
            if status == CodeStatus::Continue {
                continue;
            } else {
                break;
            }
        }
    }
    if let Some(true) = guess_code_param.fast {
        wait_code_status(&init_data, guess_code_param.chat_id).await;
    }
    tracing::info!("关闭聊天");
    functions::close_chat(guess_code_param.chat_id, client_id).await.unwrap();
    anyhow::Ok(())
}

#[derive(PartialEq)]
enum CodeStatus {
    Continue,
    End,
}

static ONCE: tokio::sync::OnceCell<anyhow::Result<()>> = tokio::sync::OnceCell::const_new();

pub async fn use_code(init_data: &InitData, code: &str, bot_id: i64, archive: Option<bool>, client_id: i32) -> anyhow::Result<()> {
    if code.contains("*") || code.contains("?") || code.contains("░") || code.contains("@") || code.contains("#") || code.contains("!") || code.contains("$") || code.contains("%") || code.contains("^") || code.contains("&") {
        return Err(anyhow::anyhow!("注册码包含非法字符"));
    }
    ONCE.get_or_init(|| async {
        tracing::info!("查找聊天");
        let mut limit = 20;
        'find_chat: loop {
            tracing::debug!("查找聊天 limit: {}", limit);
            let chats = if let Some(true) = archive {
                functions::get_chats(Some(enums::ChatList::Archive), limit, client_id).await
            } else {
                functions::get_chats(None, limit, client_id).await
            };
            if chats.is_err() {
                return Err(anyhow!("获取聊天列表失败: {:?}", chats.as_ref().err()));
            }
            let enums::Chats::Chats(chats) = chats.unwrap();
            if chats.chat_ids.len() < limit as usize && limit > 20 {
                return Err(anyhow!("未找到ID为 {} 的聊天", bot_id));
            }
            for chat_id in chats.chat_ids {
                if chat_id == bot_id {
                    break 'find_chat;
                }
            }
            limit += 20;
        }
        tracing::info!("打开聊天");
        functions::open_chat(bot_id, client_id).await.unwrap();
        anyhow::Ok(())
    }).await;

    tracing::info!("发送消息");
    let message = functions::send_message(bot_id, 0, None, None, 
        enums::InputMessageContent::InputMessageText(types::InputMessageText {
            text: types::FormattedText {
                text: format!("/start {}", code.replace(" ", "")),
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
    let status = wait_code_status(&init_data, bot_id).await;
    if status == CodeStatus::Continue {
        return Err(anyhow::anyhow!("注册码已被使用"));
    }
    anyhow::Ok(())
}

async fn wait_code_status(init_data: &InitData, chat_id: i64) -> CodeStatus {
    // 等待回复
    while let Some((new_msg, update_msg)) = init_data.msg_rx.write().await.recv().await {
        if let Some(new_msg) = new_msg {
            if new_msg.message.chat_id == chat_id {
                tracing::info!("监听新消息: {} {:?} {:?}", new_msg.message.id, new_msg.message.content, new_msg.message.reply_markup);
                let content = match &new_msg.message.content {
                    enums::MessageContent::MessageText(msg) => msg.text.text.clone(),
                    enums::MessageContent::MessagePhoto(msg) => msg.caption.text.clone(),
                    enums::MessageContent::MessageAudio(msg) => msg.caption.text.clone(),
                    enums::MessageContent::MessageDocument(msg) => msg.caption.text.clone(),
                    enums::MessageContent::MessageVideo(msg) => msg.caption.text.clone(),
                    _ => "暂不支持的消息类型".to_string(),
                };
                if content.starts_with("/start") {
                    tracing::info!("消息 {} 发送成功", new_msg.message.id);
                    continue;
                }
                if content.contains("请确认好重试") {
                    return CodeStatus::Continue;
                } else if content.contains("注册码已被使用") {
                    return CodeStatus::End;
                } else if content.contains("您已进入注册状态") || content.contains("邀请注册资格") {
                    return CodeStatus::End;
                } else {
                    continue;
                }
            }
        } else if let Some(update_msg) = update_msg {
            if update_msg.chat_id == chat_id {
                tracing::info!("监听到消息内容变更: {} {:?}", update_msg.message_id, update_msg.new_content);
            }
        }
    }
    return CodeStatus::End;
}