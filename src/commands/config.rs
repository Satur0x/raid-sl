use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::framework::standard::{
    Args,
    ArgError,
    CommandResult,
    CommandOptions,
    Reason,
    macros::{command, check},
};
use serenity::futures::prelude::*;
use serenity::collector::reaction_collector::ReactionAction;
use super::{
    Conversation,
    ConfigValuesData,
    CHECK_EMOJI,
    CROSS_EMOJI,
    DEFAULT_TIMEOUT,
};
use crate::db;
use std::{
    sync::Arc,
    collections::{
        HashMap,
        HashSet,
    },
};

type BoxResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

// --- Manager Guild Check ---
#[check]
#[name = "manager_guild"]
async fn manager_guild_check(ctx: &Context, msg: &Message, _: &mut Args, _: &CommandOptions) -> Result<(), Reason> {

    let msg_guild_id = match msg.guild_id {
        None => {
            return Err(Reason::Log("Manager command outside of manager guild".to_string()));
        },
        Some(g) => g,
    };

    let manager_guild_id = {
        ctx.data.read().await.get::<ConfigValuesData>().unwrap().manager_guild_id
    };

    if msg_guild_id != manager_guild_id {
        return Err(Reason::Log("Manager command outside of manager guild".to_string()));
    }

    Ok(())
}

struct RoleEmoji {
    role: db::models::Role,
    emoji: Emoji,
}

/// Returns a Hashmap of of Emojis and Roles that overlap with EmojiId as key
async fn role_emojis(ctx: &Context, roles: Vec<db::models::Role>)
    -> BoxResult<HashMap<EmojiId, RoleEmoji>> {

        let mut map = HashMap::new();
        let emojis_guild_id = ctx.data.read().await.get::<ConfigValuesData>().unwrap().manager_guild_id;
        let emoji_guild = Guild::get(ctx, emojis_guild_id).await?;
        let emojis = emoji_guild.emojis;

        for r in roles {
            if let Some(e) = emojis.get(&EmojiId::from(r.emoji as u64)) {
                let role_emoji = RoleEmoji {
                    role: r,
                    emoji: e.clone(),
                };
                map.insert(e.id, role_emoji);
            }
        }

        Ok(map)
}

#[command]
#[checks(manager_guild)]
pub async fn add_role(ctx: &Context, msg: &Message, mut _args: Args) -> CommandResult {

    let mut role_name = String::new();
    let mut role_repr = String::new();

    let conv = Conversation::start(ctx, &msg.author).await?;
    // Ask for Role name
    conv.chan.say(ctx, format!("{}\n{}",
            "Please enter the full name of the Role",
            "Example: Power DPS"
            )).await?;

    // Get role name
    if let Some(reply) = conv.await_reply(ctx).await {
        role_name.push_str(&reply.content);
        reply.react(ctx, ReactionType::from(CHECK_EMOJI)).await?;
    } else {
        conv.timeout_msg(ctx).await?;
        return Ok(());
    }

    // Ask for repr
    conv.chan.say(ctx, format!("{}\n{}",
            "Please enter the short representation for the role (no spaces allowed)",
            "Example: pdps"
            )).await?;

    // Get repr
    let mut replies = conv.await_replies(ctx).await;
    loop {
        if let Some(reply) = replies.next().await {
            if reply.content.contains(" ") {
                conv.chan.say(ctx, "I said no spaces!!!!\nTry again:").await?;
            } else {
                role_repr.push_str(&reply.content);
                reply.react(ctx, CHECK_EMOJI).await?;
                break;
            }
        } else {
            conv.timeout_msg(ctx).await?;
            return Ok(());
        }
    }

    let mut msg = conv.chan.say(ctx, "Loading available emojis....").await?;

    // load all roles from db
    let roles = db::get_roles(&db::connect())?;
    let db_emojis: Vec<EmojiId> = roles.iter()
        .map(|r| {
            EmojiId::from(r.emoji as u64)
        })
        .collect();

    // load all roles from discord guild
    let gid = ctx.data.read().await
        .get::<ConfigValuesData>()
        .unwrap()
        .manager_guild_id;
    let emoji_guild = Guild::get(ctx, gid).await?;

    // Remove already used emojis
    let available: Vec<Emoji> = emoji_guild.emojis.values()
        .cloned()
        .filter(|e| {
            !db_emojis.contains(&e.id)
        })
        .collect();

    if available.is_empty() {
        conv.abort(ctx, Some("No more emojis for roles available")).await?;
        return Ok(());
    }

    // Present all available emojis
    for e in available.clone() {
        msg.react(ctx, ReactionType::from(e)).await?;
    }

    // Ask for emoji to represent role
    msg.edit(ctx, |m| {
        m.content("Please react to this message with the emoji to represent this role (has to be a guild emoji)")
    }).await?;

    // Wait for emoji
    let emoji = msg.await_reaction(ctx)
        .timeout(DEFAULT_TIMEOUT)
        .filter(move |r| {
            match r.emoji {
                ReactionType::Custom {animated:_, id, name:_} => {
                    available.iter().map( |e| {
                        e.id
                    }).collect::<Vec<EmojiId>>()
                    .contains(&id)
                },
                _ => false,
            }
        }).await;

    let emoji_id = match emoji {
        None => {
            conv.timeout_msg(ctx).await?;
            return Ok(());
        },
        Some(r) => {
            match r.as_inner_ref().emoji {
                ReactionType::Custom {animated:_, id, name:_} => id,
                _ => return Ok(()), // Should never occur since filtered already
            }
        }
    };

    let msg = conv.chan.send_message(ctx, |m| {
        m.embed(|e| {
            e.title("Summary");
            e.field("Full Role Name", &role_name, false);
            e.field("Representing name", &role_repr, false);
            e.field("Role Emoji", &emoji_id, false);
            e.footer(|f| {
                f.text(format!("React with {} to add the role to the database or with {} to abort",
                               CHECK_EMOJI,
                               CROSS_EMOJI,
                               ))
            });
            e
        });
        m
    }).await?;

    msg.react(ctx, CHECK_EMOJI).await?;
    msg.react(ctx, CROSS_EMOJI).await?;

    let react = msg.await_reaction(ctx).filter( |r| {
        r.emoji == ReactionType::from(CHECK_EMOJI) || r.emoji == ReactionType::from(CROSS_EMOJI)
    }).timeout(DEFAULT_TIMEOUT).await;

    if let Some(e) = react {
        if e.as_inner_ref().emoji == ReactionType::from(CHECK_EMOJI) {
            // Save to database
            let res = {
                let db_conn = db::connect();
                db::add_role(&db_conn, &role_name, &role_repr, *emoji_id.as_u64())
            };
            match res {
                Ok(_) => {
                    conv.chan.say(ctx, "Role added to database").await?;
                }
                Err(e) => {
                    conv.chan.say(ctx, format!("Error adding role to database:\n{}", e)).await?;
                }
            }
        }
    } else {
        conv.timeout_msg(ctx).await?;
        return Ok(());
    }

    Ok(())
}

#[command]
#[checks(manager_guild)]
pub async fn list_roles(ctx: &Context, msg: &Message, mut _args: Args) -> CommandResult {

    let roles = db::get_roles(&db::connect())?;

    msg.channel_id.send_message(ctx, |m| {
        m.embed( |e| {
            e.title("Roles");
            for r in roles {
                e.field(
                    format!("{} {}", Mention::from(EmojiId::from(r.emoji as u64)), r.repr),
                    r.title,
                    true);
            }
            e
        })
    }).await?;

    Ok(())

}

#[command]
#[checks(manager_guild)]
pub async fn rm_role(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {

    let role = match args.single::<String>() {
        Ok(r) => r,
        Err(_) => {
            msg.reply(ctx, "Usage example: rm_role pdps").await?;
            return Ok(());
        }
    };

    let res = db::rm_role(&db::connect(), &role);
    match res {
        Ok(1) => {msg.react(ctx, ReactionType::from(CHECK_EMOJI)).await?;},
        Ok(0) => {msg.reply(ctx, "No role deleted. Check spelling").await?;},
        Err(e) => {msg.reply(ctx, format!("{}",e)).await?;},
        _ => {msg.reply(ctx, "Multiple roles deleted. This is unexpected behavior!!!").await?;},
    }

    Ok(())
}

/* --- Trainings ---*/
const ADD_TRAINING_USAGE: &str = "Usage example: add_training \"Beginner Training\" 2015-09-18T23:56:04";

// Helper function to  update add_training embed message
async fn update_add_training(ctx: &Context, msg: &mut Message, role_emojis: &HashMap<EmojiId, RoleEmoji>, selected: &HashSet<EmojiId>, training_name: &str, training_time: &chrono::NaiveDateTime) -> BoxResult<()> {

    msg.edit(ctx, |m| {
        m.embed(|e| {
            e.description("New Training");
            e.field(
                "Details",
                format!("{}\n{}", training_name, training_time),
                false);

            for (k, v) in role_emojis.iter() {
                e.field(
                    format!("{} {}",
                            if selected.contains(k) {CHECK_EMOJI} else {CROSS_EMOJI},
                            v.role.repr),
                    format!("{} {}",
                            Mention::from(&v.emoji),
                            v.role.title),
                    true);

            }
            e.footer( |f| {
                f.text(format!("Select roles. Use {} to finish and {} to abort", CHECK_EMOJI, CROSS_EMOJI))
            });
            e
        })
    }).await?;

    Ok(())
}

#[command]
#[checks(manager_guild)]
pub async fn add_training(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {

    let training_name = match args.single_quoted::<String>() {
        Ok(r) => r,
        Err(_) => {
            msg.reply(ctx, ADD_TRAINING_USAGE).await?;
            return Ok(());
        }
    };

    let training_time = match args.single_quoted::<chrono::NaiveDateTime>() {
        Ok(r) => r,
        Err(e) => {
            match e {
                ArgError::Parse(_) => {
                    msg.reply(ctx, "Failed to parse date. Required Format: %Y-%m-%dT%H:%M:%S%").await?;
                }
                _ => {
                    msg.reply(ctx, ADD_TRAINING_USAGE).await?;
                }
            }
            return Ok(());
        }
    };

    let mut msg = msg.channel_id.send_message(ctx, |m| {
        m.embed(|e| {
            e.description("New Training");
            e.field(
                "Details",
                format!("{}\n{}", training_name, training_time),
                false);
            e.footer( |f| {f.text("Loading roles ...")});
            e
        })
    }).await?;

    // Get roles and turn them into a HashMap with Emojis
    let roles = {
        let conn = db::connect();
        db::get_roles(&conn)?
    };
    let re = Arc::new( role_emojis(ctx, roles).await? );
    // Keep track of what roles are selected by EmojiId
    let mut selected: HashSet<EmojiId> = HashSet::new();

    msg.react(ctx, CHECK_EMOJI).await?;
    msg.react(ctx, CROSS_EMOJI).await?;

    for i in re.values() {
        msg.react(ctx, i.emoji.clone()).await?;
    }

    update_add_training(ctx, &mut msg, &re, &selected, &training_name, &training_time).await?;

    // Create another reference so that it can be moved to filter function
    let collect_re = re.clone();
    let mut reacts = msg.await_reactions(ctx)
        .removed(true)
        .timeout(DEFAULT_TIMEOUT)
        .filter(move |r| {
            if r.emoji == ReactionType::from(CHECK_EMOJI) || r.emoji == ReactionType::from(CROSS_EMOJI) {
                return true;
            }
            match r.emoji {
                ReactionType::Custom { animated:_, id, name:_ } => collect_re.contains_key(&id),
                _ => false,
        }
    }).await;

    loop {
        match reacts.next().await {
            Some(r) => {
                match r.as_ref() {
                    ReactionAction::Added(r) => {
                        if r.emoji == ReactionType::from(CHECK_EMOJI) {
                            println!("CHECK: {}", r.emoji);
                            break;
                        }
                        else if r.emoji == ReactionType::from(CROSS_EMOJI) {
                            msg.reply(ctx, "Aborted").await?;
                            return Ok(());
                        }
                        match r.emoji {
                            ReactionType::Custom { animated:_, id, name:_ } => {selected.insert(id);},
                            _ => (),
                        }
                    },
                    ReactionAction::Removed(r) => {
                        match r.emoji {
                            ReactionType::Custom { animated:_, id, name:_ } => {selected.remove(&id);},
                            _ => (),
                        }
                    },
                }
                update_add_training(ctx, &mut msg, &re, &selected, &training_name, &training_time).await?;
            },
            None => {
                msg.reply(ctx, "Timed out").await?;
                return Ok(());
            }
        }
    }

    // Do all the database stuff
    let training = {
        let conn = db::connect();
        let training = db::add_training(&conn, &training_name, &training_time);
        let training = match training {
            Err(e) => {
                msg.reply(ctx, format!("{}", e)).await?;
                return Ok(())
            },
            Ok(t) => t,
        };

        for r in re.values() {
            if selected.contains(&r.emoji.id) {
                let training_role = training.add_role(&conn, r.role.id);
                match training_role{
                    Err(e) => {
                        msg.reply(ctx, format!("{}", e)).await?;
                        return Ok(());
                    },
                    _ => (),
                }
            }
        }
        training
    };

    msg.channel_id.send_message(ctx, |m| {
        m.embed(|e| {
            e.description("Training added");
            e.field("Name", training.title, false);
            e.field("Id", training.id, false);
            e.field("Date", training.date, false);
            e.field("Open", training.open, false);
            e
        });
        m
    }).await?;

    Ok(())
}
