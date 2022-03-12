use std::str::FromStr;

use serenity::{
    builder::{CreateApplicationCommand, CreateApplicationCommandPermissions},
    client::Context,
    model::interactions::application_command::{
        ApplicationCommand, ApplicationCommandInteraction, ApplicationCommandPermissionType,
    },
};

use tracing::error;

use crate::data::ConfigValues;

#[derive(Debug)]
pub struct SlashCommandParseError(String);

impl std::fmt::Display for SlashCommandParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unknown slash command: {}", self.0)
    }
}

impl std::error::Error for SlashCommandParseError {}

mod config;
mod register;
mod raid;
mod raid_boss;
mod raid_role;
mod raid_tier;

/// All slash commands
#[derive(Debug)]
pub enum AppCommands {
    Register,
    Unregister,
    Raid,
    RaidBoss,
    RaidRole,
    RaidTier,
    Config,
}

/// All commands that should be created when the bot starts
const DEFAULT_COMMANDS: [AppCommands; 7] = [
    AppCommands::Register,
    AppCommands::Unregister,
    AppCommands::Raid,
    AppCommands::RaidBoss,
    AppCommands::RaidRole,
    AppCommands::RaidTier,
    AppCommands::Config,
];

impl FromStr for AppCommands {
    type Err = SlashCommandParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            register::CMD_REGISTER => Ok(Self::Register),
            register::CMD_UNREGISTER => Ok(Self::Unregister),
            raid::CMD_RAID => Ok(Self::Raid),
            raid_boss::CMD_RAID_BOSS => Ok(Self::RaidBoss),
            raid_role::CMD_RAID_ROLE => Ok(Self::RaidRole),
            raid_tier::CMD_RAID_TIER => Ok(Self::RaidTier),
            config::CMD_CONFIG => Ok(Self::Config),
            _ => Err(SlashCommandParseError(s.to_owned())),
        }
    }
}

impl AppCommands {
    pub fn create(&self) -> CreateApplicationCommand {
        match self {
            Self::Register => register::create_reg(),
            Self::Unregister => register::create_unreg(),
            Self::Raid => raid::create(),
            Self::RaidBoss => raid_boss::create(),
            Self::RaidRole => raid_role::create(),
            Self::RaidTier => raid_tier::create(),
            Self::Config => config::create(),
        }
    }

    pub fn create_default() -> Vec<CreateApplicationCommand> {
        DEFAULT_COMMANDS
            .iter()
            .map(Self::create)
            .collect::<Vec<_>>()
    }

    pub fn permission(
        &self,
        ac: &ApplicationCommand,
        conf: &ConfigValues,
    ) -> CreateApplicationCommandPermissions {
        let mut perms = CreateApplicationCommandPermissions::default();
        perms.id(ac.id.0);

        // Here are all the configurations for Slash Command Permissions
        match self {
            Self::Raid
            | Self::RaidBoss
            | Self::RaidRole
            | Self::RaidTier
            | Self::Config => perms.create_permissions(|p| {
                p.permission(true)
                    .kind(ApplicationCommandPermissionType::Role)
                    .id(conf.squadmaker_role_id.0)
            }),
            Self::Register | Self::Unregister => perms.create_permissions(|p| {
                p.permission(true)
                    .kind(ApplicationCommandPermissionType::Role)
                    .id(conf.main_guild_id.0) // Guild id is same as @everyone
            }),
        };

        perms
    }

    async fn handle(&self, ctx: &Context, aci: &ApplicationCommandInteraction) {
        match self {
            Self::Register => register::handle_reg(ctx, aci).await,
            Self::Unregister => register::handle_unreg(ctx, aci).await,
            Self::Raid => raid::handle(ctx, aci).await,
            Self::RaidBoss => raid_boss::handle(ctx, aci).await,
            Self::RaidRole => raid_role::handle(ctx, aci).await,
            Self::RaidTier => raid_tier::handle(ctx, aci).await,
            Self::Config => config::handle(ctx, aci).await,
        }
    }
}

pub async fn slash_command_interaction(ctx: &Context, aci: &ApplicationCommandInteraction) {
    // Consider reworking to aci.data.id
    match AppCommands::from_str(&aci.data.name) {
        Ok(cmd) => cmd.handle(ctx, aci).await,
        Err(e) => error!("{}", e),
    }
}

pub mod helpers {
    use std::collections::HashMap;

    use serde_json::Value;
    use serenity::model::interactions::application_command::ApplicationCommandInteractionDataOption;

    /// Helps to quickly access commands
    pub fn command_map(opt: &ApplicationCommandInteractionDataOption) -> HashMap<String, Value> {
        opt.options
            .iter()
            .filter_map(|o| o.value.as_ref().map(|v| (o.name.clone(), v.clone())))
            .collect()
    }
}
