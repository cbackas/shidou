use anyhow::Context;

use super::env::GuildID;

pub struct DiscordUser {
    pub id: String,
    pub username: String,
    pub guilds: Vec<PartialGuild>,
}

impl DiscordUser {
    pub fn has_guild(&self, guild_id: GuildID) -> bool {
        self.guilds.iter().any(|guild| guild.id == guild_id)
    }

    pub fn has_any_guild(&self, guiild_ids: &[GuildID]) -> bool {
        guiild_ids
            .iter()
            .any(|guild_id| self.has_guild(guild_id.clone()))
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct PartialDiscordUser {
    pub id: String,
    pub username: String,
}

pub type Guilds = Vec<PartialGuild>;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct PartialGuild {
    pub id: String,
    pub name: String,
}

pub async fn get_user_info_by_token(token: &str) -> anyhow::Result<DiscordUser> {
    let client = reqwest::Client::new();

    let user_info = client
        .get("https://discord.com/api/users/@me")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .context("failed to fetch user info from discord")?
        .json::<PartialDiscordUser>()
        .await
        .context("failed to parse user info into structs")?;

    let guilds = client
        .get("https://discord.com/api/users/@me/guilds")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .context("failed to fetch guilds from discord")?
        .json::<Guilds>()
        .await
        .context("failed to parse guilds into structs")?;

    Ok(DiscordUser {
        id: user_info.id,
        username: user_info.username,
        guilds,
    })
}
