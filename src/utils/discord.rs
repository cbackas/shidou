use anyhow::Context;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct PartialDiscordUser {
    pub id: String,
    pub username: String,
}

pub async fn get_user_info_by_token(token: &str) -> anyhow::Result<PartialDiscordUser> {
    let client = reqwest::Client::new();

    let user_info = client
        .get("https://discord.com/api/users/@me")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .context("failed to get user info")?
        .json::<PartialDiscordUser>()
        .await
        .context("failed to parse user info")?;

    Ok(user_info)
}
