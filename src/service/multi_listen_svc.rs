use anyhow::anyhow;
use serde_json::json;
use tdlib_rs::{functions, enums};

use crate::{config::args_conf::MultiListenArgs, util::http_util};

use super::init_svc::{InitData, SimpleMessage};

pub async fn listen(init_data: InitData, listen_param: MultiListenArgs) -> anyhow::Result<()> {
    let client_id = init_data.client_id;
    // 需要先把聊天找到，才能监听聊天
    tracing::info!("查找聊天");
    let mut not_find = listen_param.chat_id.clone();
    let mut limit = 20;
    'find_chat: loop {
        tracing::debug!("查找聊天 limit: {}", limit);
        let chats = functions::get_chats(None, limit, client_id).await;
        if chats.is_err() {
            return Err(anyhow!("获取聊天列表失败: {:?}", chats.as_ref().err()));
        }
        let enums::Chats::Chats(chats) = chats.unwrap();
        for chat_id in &chats.chat_ids {
            if let Ok(index) = not_find.binary_search(chat_id) {
                tracing::info!("打开聊天 {}", chat_id);
                functions::open_chat(chat_id.clone(), client_id).await.unwrap();
                not_find.remove(index);
                if not_find.len() == 0 {
                    break 'find_chat;
                }
            }
        }
        if chats.chat_ids.len() < limit as usize && limit > 20 {
            return Err(anyhow!("未找到ID为 {:?} 的聊天", not_find));
        }
        limit += 20;
    }
    tracing::info!("监听消息");
    while let Some((new_msg, new_content)) = init_data.msg_rx.write().await.recv().await {
        let msg = if new_msg.is_some() {
            let msg = new_msg.unwrap();
            SimpleMessage {
                id: msg.message.id,
                chat_id: msg.message.chat_id,
                content: msg.message.content,
            }
        } else {
            let msg = new_content.unwrap();
            SimpleMessage {
                id: msg.message_id,
                chat_id: msg.chat_id,
                content: msg.new_content,
            }
        };
        if listen_param.chat_id.contains(&msg.chat_id) {
            tracing::info!("监听消息: {} {:?}", msg.id, msg.content);
            let res = webhook(&msg, &listen_param.webhook_url).await;
            if res.is_err() {
                return Err(anyhow!("webhook 消息失败 {:?}", res.err()));
            }
        }
    }
    for chat_id in listen_param.chat_id {
        tracing::info!("关闭聊天 {}", chat_id);
        functions::close_chat(chat_id, client_id).await.unwrap();
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

// 文件下载
// let file_id = "AgACAgEAAxkBAAIDX2aUkPfcOr-5W9A-l0ub3QbOU7ZXAALdrTEbiLyRRNbA4ldm07A5AQADAgADbQADNQQ".to_string();
// let file = functions::get_remote_file(file_id, None, client_id).await;
// if file.is_err() {
//     tracing::error!("{:?}", file);
// } else {
//     let enums::File::File(file) = file.unwrap();
//     let file = functions::download_file(file.id, 1, 0, 0, true, client_id).await;
//     if file.is_err() {
//         tracing::error!("{:?}", file);
//     } else {
//         let enums::File::File(file) = file.unwrap();
//         tracing::info!("{:?}", file.local.path);
//     }
// }