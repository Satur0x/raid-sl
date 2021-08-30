use serenity::framework::standard::{
    macros::{command, group},
    Args, CommandResult,
};
use serenity::model::interactions::message_component::ButtonStyle;
use serenity::model::interactions::InteractionResponseType;
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::components::*;
use crate::db;
use crate::log::*;
use crate::utils::{self, ALARM_CLOCK_EMOJI, DEFAULT_TIMEOUT};

use std::collections::HashSet;

#[group]
#[commands(ping, dudu, button, role_button, role_select, multi_embed, log_test)]
struct Misc;

#[command]
pub async fn ping(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    LogResult::command(ctx, msg, || async {
        msg.channel_id.say(&ctx.http, "pong").await?;
        LogResult::Ok(LogAction::LogOnly("pong".into()))
    })
    .await
}

#[command]
pub async fn dudu(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    LogResult::command(ctx, msg, || async {
        msg.channel_id.say(&ctx.http, "BONK").await?;
        LogResult::Ok(LogAction::LogOnly("BONK".into()))
    })
    .await
}

#[command]
pub async fn button(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    LogResult::command(ctx, msg, || async {
        let mut msg = msg
            .channel_id
            .send_message(ctx, |m| {
                m.content("Uhhhh look. Fancy buttons =D");
                m.components(|c| {
                    c.create_action_row(|ar| {
                        ar.create_button(|b| {
                            b.style(ButtonStyle::Primary);
                            b.label("Hello");
                            b.custom_id("hello")
                        });
                        ar.create_button(|b| {
                            b.style(ButtonStyle::Primary);
                            b.label("Crossroads");
                            b.custom_id("crossroads")
                        });
                        ar.create_button(|b| {
                            b.style(ButtonStyle::Primary);
                            b.label("Inn");
                            b.custom_id("inn")
                        });
                        ar
                    });
                    c.add_action_row(confirm_abort_action_row());
                    c
                })
            })
            .await?;

        let i = msg
            .await_component_interaction(ctx)
            .timeout(DEFAULT_TIMEOUT)
            .await;

        match i {
            None => {
                msg.edit(ctx, |m| {
                    m.content(&format!("Timed out {}", ALARM_CLOCK_EMOJI));
                    m.components(|c| c);
                    m
                })
                .await?;
                LogResult::Ok(LogAction::LogOnly(format!("Timed out")))
            }
            Some(i) => {
                i.create_interaction_response(ctx, |r| {
                    r.kind(InteractionResponseType::UpdateMessage);
                    r.interaction_response_data(|d| {
                        d.content(format!("You clicked: {}", resolve_button_response(&i)));
                        d.components(|c| c)
                    })
                })
                .await?;
                LogResult::Ok(LogAction::LogOnly(format!("Clicked {}", &i.data.custom_id)))
            }
        }
    })
    .await
}

#[command]
pub async fn role_button(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    LogResult::command(ctx, msg, || async {
        let roles = db::Role::all_active(ctx).await?;
        msg.channel_id
            .send_message(ctx, |m| {
                m.content("Here are all role buttons");
                m.components(|c| {
                    c.set_action_rows(role_action_row(&roles));
                    c
                });
                m
            })
            .await?;
        LogResult::Ok(LogAction::None)
    })
    .await
}

#[command]
pub async fn role_select(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    LogResult::command(ctx, msg, || async {
        let roles = db::Role::all_active(ctx).await?;
        let selected: HashSet<String> = HashSet::new();

        let mut m = msg
            .channel_id
            .send_message(ctx, |m| {
                m.add_embed(|e| e.field("This is an initial embed", "Ignore this", false))
            })
            .await?;

        utils::select_roles(ctx, &mut m, &msg.author, &roles, selected).await?;
        LogResult::Ok(LogAction::None)
    })
    .await
}

#[command]
pub async fn multi_embed(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    LogResult::command(ctx, msg, || async {
        msg.channel_id
            .send_message(ctx, |m| {
                m.content("Sending message with multiple embeds");
                m.add_embed(|e| {
                    e.description("Embed numer 1");
                    e.field("Embed", "one", false);
                    e
                });
                m.add_embed(|e| {
                    e.description("Embed numer 2");
                    e.field("Embed", "two", false);
                    e
                });
                m.add_embed(|e| {
                    e.description("Embed numer 3");
                    e.field("Embed", "three", false);
                    e
                });
                m
            })
            .await?;
        LogResult::Ok(LogAction::None)
    })
    .await
}

#[command]
pub async fn log_test(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    log_command(ctx, msg, || async {
        msg.channel_id
            .send_message(ctx, |m| m.content("Testing logs, hmmmm"))
            .await?;
        let cmd = args.single::<String>()?;
        match cmd.as_str() {
            "ok" => Ok("Everything ok".into()),
            "num" => {
                let num = args.single::<i32>().log_reply(msg)?;
                Ok(num.to_string())
            }
            _ => Ok("Test".to_string()),
        }
    })
    .await
}
