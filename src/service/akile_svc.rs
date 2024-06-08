use anyhow::anyhow;

use crate::config::args_conf::ChatArgs;

pub async fn checkin(client_id: i32) -> anyhow::Result<()> {
    let akile_chat_id = std::env::var("AKILE_CHAT_ID");
    if akile_chat_id.is_err() {
        return Err(anyhow!("AKILE_CHAT_ID 环境变量配置错误，跳过 akile 签到， AKILE_CHAT_ID: {:?}", akile_chat_id));
    }
    let akile_chat_id = akile_chat_id.unwrap().parse().unwrap();
    super::chat_svc::chat(client_id, ChatArgs {
        chat_id: akile_chat_id,
        message: "/checkin@akilecloud_bot".to_string(),
    }).await
}