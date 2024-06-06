use tdlib::functions;

pub async fn checkin(client_id: i32) {
    let akile_chat_id = std::env::var("AKILE_CHAT_ID");
    if akile_chat_id.is_err() {
        tracing::info!("AKILE_CHAT_ID 环境变量配置错误，跳过 akile 签到， AKILE_CHAT_ID: {:?}", akile_chat_id);
        return;
    }
    let akile_chat_id = akile_chat_id.unwrap().parse().unwrap();
    functions::open_chat(akile_chat_id, client_id).await.unwrap();
    let message = functions::send_message(akile_chat_id, 0, None, None, None,
        tdlib::enums::InputMessageContent::InputMessageText(tdlib::types::InputMessageText {
            text: tdlib::types::FormattedText {
                text: "/checkin@akilecloud_bot".to_string(),
                entities: vec![]
            },
            disable_web_page_preview: true,
            clear_draft: true
        }), client_id).await;
    if message.is_err() {
        tracing::error!("发送 akile 签到失败: {:?}", message.as_ref().err());
        return;
    }
    let tdlib::enums::Message::Message(message) = message.unwrap();
    tracing::info!("发送 akile 签到消息 id: {} content: {:?}", message.id, message.content);
    functions::close_chat(akile_chat_id, client_id).await.unwrap();
}