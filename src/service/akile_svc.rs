use anyhow::anyhow;

use crate::config::args_conf::ChatArgs;

use super::init_svc::InitData;

pub async fn checkin(init_data: InitData) -> anyhow::Result<()> {
    let akile_chat_id = std::env::var("AKILE_CHAT_ID");
    if akile_chat_id.is_err() {
        return Err(anyhow!("AKILE_CHAT_ID 环境变量配置错误，跳过 akile 签到， AKILE_CHAT_ID: {:?}", akile_chat_id));
    }
    let akile_chat_id = akile_chat_id.unwrap().parse().unwrap();
    super::chat_svc::chat(init_data.clone(), ChatArgs {
        chat_id: akile_chat_id,
        message: "/checkin@akilecloud_bot".to_string(),
    }).await?;
    super::chat_svc::chat(init_data, ChatArgs {
        chat_id: akile_chat_id,
        message: "煞笔6b".to_string(),
    }).await
}