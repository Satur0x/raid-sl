use crate::commands::*;
use crate::db;
use serenity::{
    prelude::*,
    client::bridge::gateway::ShardMessenger,
    collector::reaction_collector::*,
    http::CacheHttp,
    model::{
        channel::{Message, Reaction, ReactionType},
        user::User,
        id::{UserId, RoleId, EmojiId},
        guild::{Guild, Emoji},
        misc::Mention,
    },
    builder::CreateEmbed,
    futures::StreamExt,
};
use std::{
    sync::Arc,
    collections::{HashMap,HashSet},
    iter::FromIterator,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub enum YesOrNo {
    Yes,
    No,
}

/// Reacts with CHECK_EMOJI and CROSS_EMOJI on the provided message
pub async fn send_yes_or_no<'a>(
    cache_http: &'a impl CacheHttp,
    msg: &'a Message,
) -> Result<(Reaction, Reaction)> {
    let check = msg.react(cache_http, CHECK_EMOJI).await?;
    let cross = msg.react(cache_http, CROSS_EMOJI).await?;
    Ok((check, cross))
}

/// Awaits the CHECK_EMOJI or CROSS_EMOJI reaction on a message using the default timeout
pub async fn await_yes_or_no<'a>(
    shard_messenger: &'a impl AsRef<ShardMessenger>,
    msg: &'a Message,
    user_id: Option<UserId>,
) -> Option<YesOrNo> {
    let collector = msg
        .await_reaction(shard_messenger)
        .timeout(DEFAULT_TIMEOUT)
        .filter(|r| {
            r.emoji == ReactionType::from(CHECK_EMOJI) || r.emoji == ReactionType::from(CROSS_EMOJI)
        });

    let collector = match user_id {
        Some(u) => collector.author_id(u),
        None => collector,
    };

    match collector.await {
        None => return None,
        Some(r) => match r.as_ref() {
            ReactionAction::Added(e) => {
                if e.emoji == ReactionType::from(CHECK_EMOJI) {
                    return Some(YesOrNo::Yes);
                }
                return Some(YesOrNo::No);
            }
            _ => return None,
        },
    }
}

/// Helper struct
pub struct RoleEmoji {
    pub role: db::Role,
    pub emoji: Emoji,
}

pub type RoleEmojiMap = HashMap<EmojiId, RoleEmoji>;

/// Returns a Hashmap of of Emojis and Roles that overlap with EmojiId as key
pub async fn role_emojis(
    ctx: &Context,
    roles: Vec<db::Role>,
) -> Result<RoleEmojiMap> {
    let mut map = HashMap::new();
    let emojis_guild_id = ctx
        .data
        .read()
        .await
        .get::<ConfigValuesData>()
        .unwrap()
        .emoji_guild_id;
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

/// Verifies if the discord user has the required tier for a training
pub async fn verify_tier(ctx: &Context, training: &db::Training, user: &User) -> Result<bool> {
    let tier = training.get_tier().await;
    let tier_mappings = match tier {
        None => return Ok(true),
        Some(t) => {
            Arc::new(t?).get_discord_roles().await?
        }
    };
    let roles_set = {
        let guild = ctx.data.read().await.get::<ConfigValuesData>().unwrap().main_guild_id;
        let roles = guild.member(ctx, user.id).await?.roles;
        HashSet::<RoleId>::from_iter(roles)
    };

    let passed = tier_mappings
        .iter()
        .any(|t| {
            roles_set.contains(&RoleId::from(t.discord_role_id as u64))
        });
    Ok(passed)
}

/// Looks at the user permissions and filters out trainings the user has not sufficient permissions
/// for
pub async fn filter_trainings(ctx: &Context, trainings: Vec<db::Training>, user: &User) -> Result<Vec<db::Training>> {
    let roles = {
        let guild = ctx.data.read().await.get::<ConfigValuesData>().unwrap().main_guild_id;
        guild.member(ctx, user.id).await?.roles
    };

    let tiers = db::Tier::all().await?;

    let mut tier_map: HashMap<i32, Vec<db::TierMapping>> = HashMap::new();

    for t in tiers {
        let t = Arc::new(t);
        let discord_roles = t.clone().get_discord_roles().await?;
        tier_map.insert(t.id, discord_roles);
    }

    Ok(trainings.into_iter().filter( |tier| {
        match tier.tier_id {
            None => true,
            Some(id) => {
                match tier_map.get(&id) {
                    None => false,
                    Some(tm_vec) => tm_vec.iter().any( |tm| {
                        roles.iter().any(|r| { *r == (tm.discord_role_id as u64) })
                    }),
                }
            }
        }
    }).collect())
}

pub fn format_training_slim(t: &db::Training) -> String {
    String::from(format!(
        "Name: `{}`\nDate `{} UTC`",
        t.title,
        t.date,
    ))
}

fn select_roles_embed(
    re_map: &HashMap<&db::Role, &Emoji>, // map with all relevant emojis
    sel: &HashSet<&db::Role>,            // already selected roles
    initial: bool,
) -> CreateEmbed {
    let mut e = CreateEmbed::default();

    e.description("Select roles");
    e.fields(re_map.iter().map(|(r, e)| {
        let des = String::from(format!(
            "{} | {}",
            if sel.contains(r) {
                CHECK_EMOJI
            } else {
                CROSS_EMOJI
            },
            r.repr
        ));
        let cont = String::from(format!("{} | {}", Mention::from(*e), r.title));
        (des, cont, true)
    }));
    e.footer(|f| {
        if initial {
            f.text("Loading emojis. Please wait...")
        } else {
            f.text(format!("React with the corresponding emoji to select/unselect a role. Use {} to confirm. Use {} to abort", CHECK_EMOJI, CROSS_EMOJI))
        }
    });
    e
}

/// Uses a conversation to select and un-select different roles
/// Selected and unselected roles are given and returned.
/// If the corresponding emoji is not found in the guild it will remain in
/// its original set
pub async fn select_roles<'a>(
    ctx: &Context,
    conv: &mut Conversation,
    mut selected: HashSet<&'a db::Role>,
    mut unselected: HashSet<&'a db::Role>,
) -> Result<(HashSet<&'a db::Role>, HashSet<&'a db::Role>)> {
    let emojis_guild_id = ctx
        .data
        .read()
        .await
        .get::<ConfigValuesData>()
        .unwrap()
        .emoji_guild_id;
    let emoji_guild = Arc::new(Guild::get(ctx, emojis_guild_id).await?);

    // Generate map of all roles with corresponding Emoji
    let mut re_map: HashMap<&db::Role, &Emoji> =
        HashMap::with_capacity(selected.len() + unselected.len());
    // for quick reverse lookup
    let mut er_map: HashMap<EmojiId, &db::Role> =
        HashMap::with_capacity(selected.len() + unselected.len());
    // Moved into closures as Arc
    let mut e_set: HashSet<EmojiId> = HashSet::with_capacity(selected.len() + unselected.len());

    selected.iter().chain(unselected.iter()).for_each(|r| {
        let e_id = EmojiId::from(r.emoji as u64);
        if let Some(e) = emoji_guild.emojis.get(&e_id) {
            re_map.insert(r, e);
            er_map.insert(e_id, r);
            e_set.insert(e_id);
        }
    });

    // No more mut
    let re_map = re_map;
    let er_map = er_map;
    let e_set = Arc::new(e_set);

    // Instantly start listening to reactions. Stream will catch them
    // and we wont miss them
    let reactor_emojis = e_set.clone();
    let mut reactor = conv
        .await_reactions(ctx)
        .added(true)
        .removed(true)
        .filter(move |r| {
            if r.emoji == ReactionType::from(CHECK_EMOJI)
                || r.emoji == ReactionType::from(CROSS_EMOJI)
            {
                return true;
            }
            match &r.emoji {
                ReactionType::Custom {
                    animated: _,
                    id,
                    name: _,
                } => reactor_emojis.contains(&id),
                _ => false,
            }
        })
        .await;

    let emb = select_roles_embed(&re_map, &selected, true);
    conv.msg
        .edit(ctx, |m| {
            m.content("");
            m.embed(|e| {
                e.0 = emb.0;
                e
            })
        })
        .await?;

    // sending emojis is slow due to discord rate limits (I assume)
    // So do it parallel while already accepting reactions
    // Need to clone some stuff or it cant be moved to another thread
    let msg_send = conv.msg.clone();
    let ctx_send = ctx.clone();
    let emoji_guild_send = emoji_guild.clone();
    tokio::spawn(async move {
        send_yes_or_no(&ctx_send, &msg_send).await.unwrap();
        for e in e_set.iter() {
            if let Some(e) = emoji_guild_send.emojis.get(e) {
                msg_send.react(&ctx_send, e.clone()).await.unwrap();
            }
        }
    });

    // Now wait for emoji reactions and update message
    loop {
        let emb = select_roles_embed(&re_map, &selected, false);
        conv.msg
            .edit(ctx, |m| {
                m.embed(|e| {
                    e.0 = emb.0;
                    e
                })
            })
            .await?;

        let react = reactor.next().await;
        println!("{:?}",react);

        let react = match react {
            None => {
                return Err(Box::new(ConversationError::TimedOut));
            }
            // Dont care if added or removed. Its toggled
            Some(e) => match *e {
                ReactionAction::Added(ref r) => r.clone(),
                ReactionAction::Removed(ref r) => r.clone(),
            },
        };

        if react.emoji == ReactionType::from(CHECK_EMOJI) {
            break;
        } else if react.emoji == ReactionType::from(CROSS_EMOJI) {
            return Err(Box::new(ConversationError::Canceled));
        }
        if let ReactionType::Custom {
            animated: _,
            id,
            name: _,
        } = &react.emoji
        {
            if let Some(role) = er_map.get(&id) { // Should never fail
                // if able to remove from selected add to unselected
                if selected.remove(role) {
                    unselected.insert(role);
                // otherwise remove from unselected and add to selected
                } else if unselected.remove(role) {
                    selected.insert(role);
                }
            }
        }
    }

    Ok((selected, unselected))
}
