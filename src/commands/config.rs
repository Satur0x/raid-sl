use super::ADMIN_ROLE_CHECK;
use crate::{data::*, db, signup_board, utils::*};
use serenity::framework::standard::{
    macros::{command, group},
    Args, CommandResult,
};
use serenity::model::prelude::*;
use serenity::prelude::*;

#[group]
#[only_in(guilds)]
#[commands(
    set_log_info,
    set_log_error,
    set_signup_board_category,
    signup_board_reset
)]
struct Config;

#[command]
#[checks(admin_role)]
#[description = "Sets the log channel for info"]
#[example = "#logs_info"]
#[usage = "channel_mention"]
#[only_in("guild")]
#[num_args(1)]
pub async fn set_log_info(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let channel_id: ChannelId = match args.single::<ChannelId>() {
        Err(_) => {
            msg.reply(ctx, "No valid channel provided").await?;
            return Ok(());
        }
        Ok(c) => c,
    };

    // save in memory
    {
        let write_lock = ctx
            .data
            .read()
            .await
            .get::<LogConfigData>()
            .unwrap()
            .clone();
        write_lock.write().await.info = Some(channel_id);
    }

    // save to db
    let conf = db::Config {
        name: String::from(INFO_LOG_NAME),
        value: channel_id.to_string(),
    };

    match conf.save().await {
        Ok(_) => (),
        Err(e) => {
            msg.reply(ctx, "Unexpected error").await?;
            return Err(e.into());
        }
    }
    msg.react(ctx, CHECK_EMOJI).await?;
    Ok(())
}

#[command]
#[checks(admin_role)]
#[description = "Sets the log channel for error"]
#[example = "#logs_error"]
#[usage = "channel_mention"]
#[only_in("guild")]
#[num_args(1)]
pub async fn set_log_error(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let channel_id: ChannelId = match args.single::<ChannelId>() {
        Err(_) => {
            msg.reply(ctx, "No valid channel provided").await?;
            return Ok(());
        }
        Ok(c) => c,
    };

    // set in memory
    {
        let write_lock = ctx
            .data
            .read()
            .await
            .get::<LogConfigData>()
            .unwrap()
            .clone();
        write_lock.write().await.error = Some(channel_id);
    }

    // save to db
    let conf = db::Config {
        name: String::from(ERROR_LOG_NAME),
        value: channel_id.to_string(),
    };

    match conf.save().await {
        Ok(_) => (),
        Err(e) => {
            msg.reply(ctx, "Unexpected error").await?;
            return Err(e.into());
        }
    }

    msg.react(ctx, CHECK_EMOJI).await?;
    Ok(())
}

#[command]
#[checks(admin_role)]
#[description = "Sets category id for the SignupBoard"]
#[usage = "category_id"]
#[only_in("guild")]
#[num_args(1)]
pub async fn set_signup_board_category(
    ctx: &Context,
    msg: &Message,
    mut args: Args,
) -> CommandResult {
    let channel_id: ChannelId = match args.single::<ChannelId>() {
        Err(_) => {
            msg.reply(ctx, "No valid channel provided").await?;
            return Ok(());
        }
        Ok(c) => c,
    };

    // set in memory
    {
        let write_lock = ctx
            .data
            .read()
            .await
            .get::<SignupBoardData>()
            .unwrap()
            .clone();
        write_lock.write().await.set_category_channel(channel_id);
    }

    // save to db
    let conf = db::Config {
        name: String::from(signup_board::SIGNUP_BOARD_NAME),
        value: channel_id.to_string(),
    };

    match conf.save().await {
        Ok(_) => (),
        Err(e) => {
            msg.reply(ctx, "Unexpected error").await?;
            return Err(e.into());
        }
    }

    msg.react(ctx, CHECK_EMOJI).await?;
    Ok(())
}

#[command]
#[checks(admin_role)]
#[description = "fully resets the Signup Board"]
#[usage = ""]
#[only_in("guild")]
#[num_args(0)]
pub async fn signup_board_reset(ctx: &Context, msg: &Message, _: Args) -> CommandResult {
    let write_lock = ctx
        .data
        .read()
        .await
        .get::<SignupBoardData>()
        .unwrap()
        .clone();

    write_lock.write().await.reset(ctx).await?;

    msg.react(ctx, CHECK_EMOJI).await?;
    Ok(())
}
