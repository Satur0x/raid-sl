use std::{borrow::Cow, collections::HashMap, time::Duration};

use super::helpers::*;
use crate::{
    data,
    db::{self, Tier, RaidState},
    embeds::{embed_add_roles, CrossroadsEmbeds},
    logging::*,
    signup_board, status,
};
use anyhow::{anyhow, bail, Context as ErrContext, Result};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use itertools::Itertools;
use serde::Serialize;
use serenity::model::{
    id::EmojiId,
    interactions::{
        application_command::{
            ApplicationCommandInteraction, ApplicationCommandInteractionDataOption,
            ApplicationCommandOptionType,
        },
        InteractionApplicationCommandCallbackDataFlags, InteractionResponseType,
    },
};
use serenity::{
    builder::{CreateApplicationCommand, CreateEmbed},
    client::Context,
    futures::future,
    futures::future::OptionFuture,
    http::AttachmentType,
    model::{
        guild::{Member, PartialGuild, Role},
        id::RoleId,
        misc::Mention,
    },
};
use serenity_tools::{
    builder::{CreateActionRowExt, CreateEmbedExt},
    collectors::MessageCollectorExt,
    components::Button,
    interactions::{ApplicationCommandInteractionExt, MessageComponentInteractionExt},
};

type MessageFlags = InteractionApplicationCommandCallbackDataFlags;

pub(super) const CMD_RAID: &str = "raid";
const CHECK_EMOJI: char = '✅';

pub fn create() -> CreateApplicationCommand {
    let mut app = CreateApplicationCommand::default();
    app.name(CMD_RAID);
    app.description("Manage raids");
    app.default_permission(false);
    app.create_option(|o| {
        o.kind(ApplicationCommandOptionType::SubCommand);
        o.name("add");
        o.description("Add a new Raid");
        o.create_sub_option(|o| {
            o.kind(ApplicationCommandOptionType::String);
            o.name("name");
            o.description("The name of the raid");
            o.required(true)
        });
        o.create_sub_option(|o| {
            o.kind(ApplicationCommandOptionType::String);
            o.name("day");
            o.description("Day in UTC. Format: yyyy-mm-dd");
            o.required(true)
        });
        o.create_sub_option(|o| {
            o.kind(ApplicationCommandOptionType::String);
            o.name("time");
            o.description("Time in UTC. Format: HH:MM:SS");
            o.required(true)
        });
        o.create_sub_option(|o| {
            o.kind(ApplicationCommandOptionType::String);
            o.name("roles");
            o.description("The roles available for the raid. Comma separated list of repr's. Example: dps,druid,qfb");
            o.required(true)
        });
        o.create_sub_option(|o| {
            o.kind(ApplicationCommandOptionType::String);
            o.name("bosses");
            o.description("The bosses available for the raid. Comma separated list of repr's. Example: vg,gorse,trio");
            o.required(true)
        });
        o.create_sub_option(|o| {
            o.kind(ApplicationCommandOptionType::String);
            o.name("tier");
            o.description("The required tier for the raid. If left empty raid is open for everyone")
        })
    });
    app.create_option(|o| {
        o.kind(ApplicationCommandOptionType::SubCommand);
        o.name("set");
        o.description("Change the state of one or multiple raid(s)");
        o.create_sub_option(|o| {
            o.kind(ApplicationCommandOptionType::String);
            o.name("state");
            o.description("The state to set to");
            o.required(true);
            o.add_string_choice("created", "created");
            o.add_string_choice("open", "open");
            o.add_string_choice("closed", "closed");
            o.add_string_choice("started", "started");
            o.add_string_choice("finished", "finished")
        });
        o.create_sub_option(|o| {
            o.kind(ApplicationCommandOptionType::String);
            o.name("day");
            o.description(
                "Select all raids from that day. Format: yyyy-mm-dd. Comma separated list",
            )
        });
        o.create_sub_option(|o| {
            o.kind(ApplicationCommandOptionType::String);
            o.name("ids");
            o.description("Select raid(s) with the specified id. Comma separated list")
        })
    });
    app.create_option(|o| {
        o.kind(ApplicationCommandOptionType::SubCommand);
        o.name("list");
        o.description("List all raids of a day basic information");
        o.create_sub_option(|o| {
            o.kind(ApplicationCommandOptionType::String);
            o.required(true);
            o.name("day");
            o.description(
                "Select all raids from that day. Format: yyyy-mm-dd. Comma separated list",
            )
        })
    });
    app.create_option(|o| {
        o.kind(ApplicationCommandOptionType::SubCommand);
        o.name("download");
        o.description("Download one or multiple raid(s)");
        o.create_sub_option(|o| {
            o.kind(ApplicationCommandOptionType::String);
            o.name("day");
            o.description(
                "Select all raids from that day. Format: yyyy-mm-dd. Comma separated list",
            )
        });
        o.create_sub_option(|o| {
            o.kind(ApplicationCommandOptionType::String);
            o.name("ids");
            o.description("Select raid(s) with the specified id. Comma separated list")
        });
        o.create_sub_option(|o| {
            o.kind(ApplicationCommandOptionType::String);
            o.name("format");
            o.description("Select the download format. Default: csv");
            o.add_string_choice("json", "json");
            o.add_string_choice("csv", "csv")
        })
    });
    app.create_option(|o| {
        o.kind(ApplicationCommandOptionType::SubCommand);
        o.name("info");
        o.description("Show detailed information about a raid");
        o.create_sub_option(|o| {
            o.kind(ApplicationCommandOptionType::Integer);
            o.required(true);
            o.name("id");
            o.description("The id of the raid");
            o.min_int_value(0)
        });
        o.create_sub_option(|o| {
            o.kind(ApplicationCommandOptionType::Boolean);
            o.name("public");
            o.description("Whether to post this public or not. Default: false")
        })
    });
    app
}

pub async fn handle(ctx: &Context, aci: &ApplicationCommandInteraction) {
    log_discord(ctx, aci, |trace| async move {
        trace.step("Parsing command");
        if let Some(sub) = aci.data.options.get(0) {
            match sub.name.as_ref() {
                "add" => add(ctx, aci, sub, trace).await,
                "set" => set(ctx, aci, sub, trace).await,
                "download" => download(ctx, aci, sub, trace).await,
                "info" => info(ctx, aci, sub, trace).await,
                "list" => list(ctx, aci, sub, trace).await,
                _ => bail!("{} not yet available", sub.name),
            }
        } else {
            bail!("Invalid command")
        }
    })
    .await;
}

async fn raids_from_days(ctx: &Context, value: &str) -> Result<Vec<db::Raid>> {
    let days: Vec<NaiveDate> = value
        .split(',')
        .map(|s| s.parse())
        .collect::<Result<Vec<_>, _>>()
        .context("Could not parse date")?;

    let raids_fut = days
        .into_iter()
        .map(|d| db::Raid::by_date(ctx, d))
        .collect::<Vec<_>>();

    Ok(future::try_join_all(raids_fut)
        .await?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>())
}

async fn raids_from_ids(ctx: &Context, value: &str) -> Result<Vec<db::Raid>> {
    let i: Vec<i32> = value
        .split(',')
        .map(|s| s.parse())
        .collect::<Result<Vec<_>, _>>()?;

    let raids_fut = i
        .into_iter()
        .map(|i| db::Raid::by_id(ctx, i))
        .collect::<Vec<_>>();

    Ok(future::try_join_all(raids_fut)
        .await
        .context("Raid id does not exist")?)
}

async fn add(
    ctx: &Context,
    aci: &ApplicationCommandInteraction,
    option: &ApplicationCommandInteractionDataOption,
    trace: LogTrace,
) -> Result<()> {
    let cmds = command_map(option);

    trace.step("Parsing basic raid data");

    let name = cmds
        .get("name")
        .and_then(|n| n.as_str())
        .context("name not set")?;

    let day: NaiveDate = cmds
        .get("day")
        .and_then(|n| n.as_str())
        .context("day not set")?
        .parse()
        .context("Could not parse date")
        .map_err_reply(|what| aci.create_quick_error(ctx, what, true))
        .await?;

    let time: NaiveTime = cmds
        .get("time")
        .and_then(|n| n.as_str())
        .context("time not set")?
        .parse()
        .context("Could not parse time")
        .map_err_reply(|what| aci.create_quick_error(ctx, what, true))
        .await?;

    let datetime: NaiveDateTime = day.and_time(time);

    let mut emb = CreateEmbed::xdefault();
    emb.title("Creating a new raid");
    emb.field("Name", name, false);
    emb.field("Date/Time", format!("<t:{}>", datetime.timestamp()), false);

    let mut emb_loading_roles = emb.clone();
    emb_loading_roles.field("Roles", "Loading...", false);
    aci.create_interaction_response(ctx, |r| {
        r.kind(InteractionResponseType::ChannelMessageWithSource);
        r.interaction_response_data(|d| {
            d.flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL);
            d.add_embed(emb_loading_roles)
        })
    })
    .await?;

    let msg = aci.get_interaction_response(ctx).await?;

    trace.step("Loading raid roles");

    let roles_str: Vec<&str> = cmds
        .get("roles")
        .and_then(|n| n.as_str())
        .context("roles not set")?
        .split(',')
        .into_iter()
        .map(|s| s.trim())
        .collect();

    let mut roles: Vec<db::Role> = Vec::with_capacity(roles_str.len());
    for r in roles_str {
        let nr = db::Role::by_repr(ctx, r.to_string())
            .await
            .with_context(|| format!("Failed to load role: {}", r))
            .map_err_reply(|what| aci.edit_quick_error(ctx, what))
            .await?;
        roles.push(nr);
    }

    embed_add_roles(&mut emb, &roles, true, false);

    let mut emb_loading_bosses = emb.clone();
    emb_loading_bosses.field("Bosses", "Loading...", false);
    aci.edit_original_interaction_response(ctx, |d| d.add_embed(emb_loading_bosses))
        .await?;

    trace.step("Loading raid bosses");

    let bosses_str: Vec<&str> = cmds
        .get("bosses")
        .and_then(|n| n.as_str())
        .context("bosses not set")?
        .split(',')
        .into_iter()
        .map(|s| s.trim())
        .collect();

    let mut bosses: Vec<db::RaidBoss> = Vec::with_capacity(bosses_str.len());
    for b in bosses_str {
        let nb = db::RaidBoss::by_repr(ctx, b.to_string())
            .await
            .with_context(|| format!("Failed to load boss {}", b))
            .map_err_reply(|what| aci.edit_quick_error(ctx, what))
            .await?;
        bosses.push(nb);
    }

    emb.fields_chunked_fmt(&bosses, |b| b.name.clone(), "Boss Pool", false, 10);

    let mut emb_loading_tier = emb.clone();
    emb_loading_tier.field("Tier", "Loading...", false);
    aci.edit_original_interaction_response(ctx, |d| d.add_embed(emb_loading_tier))
        .await?;

    trace.step("Loading tier");
    let tier_fut: OptionFuture<_> = cmds
        .get("tier")
        .and_then(|v| v.as_str())
        .map(|t| Tier::by_name(ctx, t.to_owned()))
        .into();

    let tier = tier_fut
        .await
        .transpose()
        .context("Failed to load tier")
        .map_err_reply(|what| aci.edit_quick_error(ctx, what))
        .await?;

    if let Some(t) = &tier {
        emb.field("Tier", &t.name, false);
    } else {
        emb.field("Tier", "Open for everyone", false);
    }
    aci.edit_original_interaction_response(ctx, |d| {
        d.add_embed(emb.clone());
        d.components(|c| c.create_action_row(|a| a.confirm_button().abort_button()))
    })
    .await?;

    trace.step("Waiting for confirm");

    if let Some(react) = msg
        .await_confirm_abort_interaction(ctx)
        .timeout(Duration::from_secs(60))
        .await
    {
        react.defer(ctx).await?;
        match react.parse_button()? {
            Button::Confirm => {
                trace.step("Confirmed. Saving raid");
                let raid =
                    db::Raid::insert(ctx, name.to_string(), datetime, tier.map(|t| t.id))
                        .await
                        .map_err_reply(|what| aci.edit_quick_error(ctx, what))
                        .await?;

                trace.step("Saving roles");
                for r in roles {
                    raid
                        .add_role(ctx, r.id)
                        .await
                        .map_err_reply(|what| aci.edit_quick_error(ctx, what))
                        .await?;
                }

                trace.step("Saving raid bosses");
                for tb in bosses {
                    raid
                        .add_raid_boss(ctx, tb.id)
                        .await
                        .map_err_reply(|what| aci.edit_quick_error(ctx, what))
                        .await?;
                }

                emb.field("Raid ID", raid.id, false);
                emb.footer(|f| f.text(format!("Raid added {}", CHECK_EMOJI)));
                aci.edit_original_interaction_response(ctx, |d| {
                    d.add_embed(emb);
                    d.components(|c| c)
                })
                .await?;
            }
            Button::Abort => {
                trace.step("Aborted");
                aci.edit_quick_info(ctx, "Aborted").await?;
            }
            _ => bail!("Unexpected interaction"),
        }
    } else {
        Err(anyhow!("Timed out"))
            .map_err_reply(|what| aci.edit_quick_info(ctx, what))
            .await?;
    }

    Ok(())
}

async fn set(
    ctx: &Context,
    aci: &ApplicationCommandInteraction,
    option: &ApplicationCommandInteractionDataOption,
    trace: LogTrace,
) -> Result<()> {
    // Get subcommands
    let cmds = option
        .options
        .iter()
        .map(|o| (o.name.clone(), o))
        .collect::<HashMap<_, _>>();

    // required and pre defined so fine to unwrap
    let state = cmds
        .get("state")
        .unwrap()
        .value
        .as_ref()
        .unwrap()
        .as_str()
        .unwrap()
        .parse::<RaidState>()
        .unwrap();

    // Although loading full raids is a bit overhead
    // it also guarantees they exist
    let mut raids: Vec<db::Raid> = Vec::new();

    if let Some(days) = cmds
        .get("day")
        .and_then(|d| d.value.as_ref())
        .and_then(|d| d.as_str())
    {
        raids.append(
            &mut raids_from_days(ctx, days)
                .await
                .map_err_reply(|what| aci.create_quick_error(ctx, what, true))
                .await?,
        );
    }

    if let Some(ids) = cmds
        .get("ids")
        .and_then(|d| d.value.as_ref())
        .and_then(|d| d.as_str())
    {
        raids.append(
            &mut raids_from_ids(ctx, ids)
                .await
                .map_err_reply(|what| aci.create_quick_error(ctx, what, true))
                .await?,
        );
    }

    if raids.is_empty() {
        Err(anyhow!("Select at least one raid"))
            .map_err_reply(|what| aci.create_quick_error(ctx, what, true))
            .await?;
    }

    trace.step("Traning(s) loaded");

    // filter out multiple
    raids.sort_by_key(|t| t.id);
    raids.dedup_by_key(|t| t.id);
    raids.sort_by_key(|t| t.date);

    let mut te = CreateEmbed::xdefault();
    te.title("Change raid state");
    te.description(format!("Setting the following raids to: **{}**", state));
    te.fields(raids.iter().map(|id| {
        (
            format!("{} | {}", id.id, id.title),
            format!("<t:{}>", id.date.timestamp()),
            true,
        )
    }));

    aci.create_interaction_response(ctx, |r| {
        r.kind(InteractionResponseType::ChannelMessageWithSource);
        r.interaction_response_data(|d| {
            d.flags(MessageFlags::EPHEMERAL);
            d.add_embed(te.clone());
            d.components(|c| c.create_action_row(|ar| ar.confirm_button().abort_button()))
        })
    })
    .await?;

    trace.step("Waiting for confirmation");
    let msg = aci.get_interaction_response(ctx).await?;
    match msg
        .await_component_interaction(ctx)
        .timeout(Duration::from_secs(60))
        .await
    {
        Some(response) => {
            match response.parse_button() {
                Ok(Button::Confirm) => {
                    trace.step("Confirmed");
                    response
                        .create_interaction_response(ctx, |r| {
                            r.kind(InteractionResponseType::UpdateMessage);
                            r.interaction_response_data(|d| {
                                d.flags(MessageFlags::EPHEMERAL);
                                d.components(|c| c)
                            })
                        })
                        .await?;

                    trace.step("Updating traning(s)");
                    let update_futs: Vec<_> = raids
                        .into_iter()
                        .map(|t| t.set_state(ctx, state.clone()))
                        .collect();
                    let _ = future::try_join_all(update_futs).await?;

                    response
                        .edit_original_interaction_response(ctx, |m| {
                            m.create_embed(|e| {
                                e.title("Raids updated");
                                e.description("Updating Signup Board and status ...")
                            })
                        })
                        .await?;

                    trace.step("Updating signup board");
                    signup_board::SignupBoard::get(ctx)
                        .await
                        .read()
                        .await
                        .update_overview(ctx, trace.clone())
                        .await?;

                    trace.step("Updating status");
                    status::update_status(ctx).await;

                    response
                        .edit_original_interaction_response(ctx, |m| {
                            m.add_embed(CreateEmbed::info_box("Everything updated"))
                        })
                        .await?;
                }
                Ok(Button::Abort) => {
                    trace.step("Aborted");
                    response
                        .create_interaction_response(ctx, |r| {
                            r.kind(InteractionResponseType::UpdateMessage);
                            r.interaction_response_data(|d| {
                                d.flags(MessageFlags::EPHEMERAL);
                                d.content("Aborted");
                                d.embeds(std::iter::empty());
                                d.components(|c| c)
                            })
                        })
                        .await?;
                }
                // Should not be possible
                _ => bail!("Unexpected interaction"),
            }
        }
        None => {
            Err(anyhow!("Timed out"))
                .map_err_reply(|w| aci.edit_followup_quick_info(ctx, &msg, w))
                .await?;
        }
    };

    Ok(())
}

#[derive(Serialize)]
enum DonwloadFormat {
    Json,
    Csv,
}

impl std::fmt::Display for DonwloadFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Csv => write!(f, "csv"),
            Self::Json => write!(f, "json"),
        }
    }
}

#[derive(Serialize)]
struct SignupData {
    user: db::User,
    member: Member,
    roles: Vec<String>, // we only need the repr here
    comment: Option<String>,
}

// since csv is all row based edit on the fly
#[derive(Serialize)]
struct SignupDataCsv<'a> {
    #[serde(rename = "Gw2 Account")]
    gw2_acc: &'a str,
    #[serde(rename = "Discord Account")]
    discord_acc: String,
    #[serde(rename = "Discord Ping")]
    discord_ping: String,
    #[serde(rename = "Raid Name")]
    raid_name: &'a str,
    #[serde(rename = "Roles")]
    roles: String,
    #[serde(rename = "Comment")]
    comment: Option<&'a str>,
}

#[derive(Serialize)]
struct RaidData {
    raid: db::Raid,
    available_roles: Vec<db::Role>,
    signups: Vec<SignupData>,
}

#[derive(Serialize)]
struct TierData {
    id: i32,
    name: String,
    includes: Vec<Role>,
}

#[derive(Serialize)]
struct DownloadData {
    output: DonwloadFormat,
    created: NaiveDateTime,
    raids: Vec<RaidData>,
    tiers: Vec<TierData>,
}

impl DownloadData {
    fn to_csv(&self) -> Vec<SignupDataCsv<'_>> {
        let mut v = Vec::new();

        for t in &self.raids {
            for s in &t.signups {
                let elem = SignupDataCsv {
                    gw2_acc: &s.user.gw2_id,
                    discord_acc: s.member.user.tag(),
                    discord_ping: Mention::from(&s.member).to_string(),
                    raid_name: &t.raid.title,
                    roles: s
                        .roles
                        .iter()
                        .map(|r| r.as_str())
                        .collect::<Vec<_>>()
                        .join(", "),
                    comment: s.comment.as_deref(),
                };

                v.push(elem);
            }
        }

        v
    }
}

async fn download(
    ctx: &Context,
    aci: &ApplicationCommandInteraction,
    option: &ApplicationCommandInteractionDataOption,
    trace: LogTrace,
) -> Result<()> {
    let guild_id = match ctx.data.read().await.get::<data::ConfigValuesData>() {
        Some(conf) => conf.main_guild_id,
        None => {
            bail!("Guild configuration could not be loaded");
        }
    };

    let guild = PartialGuild::get(ctx, guild_id).await?;

    // Get subcommands
    let cmds = option
        .options
        .iter()
        .map(|o| (o.name.clone(), o))
        .collect::<HashMap<_, _>>();

    // Although loading full raids is a bit overhead
    // it also guarantees they exist
    let mut raids: Vec<db::Raid> = Vec::new();

    trace.step("Loading raid's by date");
    if let Some(days) = cmds
        .get("day")
        .and_then(|d| d.value.as_ref())
        .and_then(|d| d.as_str())
    {
        raids.append(
            &mut raids_from_days(ctx, days)
                .await
                .map_err_reply(|what| aci.create_quick_error(ctx, what, true))
                .await?,
        );
    }

    trace.step("Loading raid's by id");
    if let Some(ids) = cmds
        .get("ids")
        .and_then(|d| d.value.as_ref())
        .and_then(|d| d.as_str())
    {
        raids.append(
            &mut raids_from_ids(ctx, ids)
                .await
                .map_err_reply(|what| aci.create_quick_error(ctx, what, true))
                .await?,
        );
    }

    if raids.is_empty() {
        Err(anyhow!("Select at least one raid"))
            .map_err_reply(|what| aci.create_quick_error(ctx, what, true))
            .await?;
    }

    trace.step("sort raid's");
    // filter out multiple
    raids.sort_by_key(|t| t.id);
    raids.dedup_by_key(|t| t.id);
    raids.sort_by_key(|t| t.date);

    // What to parse to
    let format = if let Some(f) = cmds
        .get("format")
        .and_then(|d| d.value.as_ref())
        .and_then(|d| d.as_str())
    {
        match f {
            "csv" => DonwloadFormat::Csv,
            "json" => DonwloadFormat::Json,
            _ => unimplemented!(),
        }
    } else {
        DonwloadFormat::Csv // Default
    };

    aci.create_quick_info(ctx, "Parsing raid data...", true)
        .await?;

    let msg = aci.get_interaction_response(ctx).await?;

    let mut log: Vec<String> = Vec::new();
    let mut tds: Vec<RaidData> = Vec::with_capacity(raids.len());

    for t in raids {
        let signups = t.get_signups(ctx).await?;
        let mut sds: Vec<SignupData> = Vec::with_capacity(signups.len());

        for s in signups {
            let user = s.get_user(ctx).await?;

            let member = match guild.member(ctx, user.discord_id()).await {
                Ok(du) => du,
                Err(_) => {
                    log.push(format!(
                        "Did not find user with id {} in discord guild. Skipped",
                        user.discord_id()
                    ));
                    continue;
                }
            };

            let roles = s
                .get_roles(ctx)
                .await?
                .into_iter()
                .map(|r| r.repr)
                .collect::<Vec<_>>();

            sds.push(SignupData {
                user,
                member,
                roles,
                comment: s.comment,
            });
        }

        let available_roles = t.all_roles(ctx).await?;

        tds.push(RaidData {
            raid: t,
            available_roles,
            signups: sds,
        });
    }

    let dbtiers = db::Tier::all(ctx).await?;
    let mut tiers: Vec<TierData> = Vec::with_capacity(dbtiers.len());

    for t in dbtiers {
        let dr = t
            .get_discord_roles(ctx)
            .await?
            .iter()
            .map(|t| guild.roles.get(&RoleId::from(t.discord_role_id as u64)))
            .collect::<Vec<_>>();

        let mut includes: Vec<Role> = Vec::with_capacity(dr.len());

        for r in dr {
            match r {
                Some(r) => includes.push(r.clone()),
                None => log.push(format!(
                    "Failed to find a discord role for tier: {}",
                    t.name
                )),
            }
        }

        tiers.push(TierData {
            id: t.id,
            name: t.name,
            includes,
        })
    }

    let data = DownloadData {
        output: format,
        created: chrono::Utc::now().naive_utc(),
        raids: tds,
        tiers,
    };

    let data_bytes = match data.output {
        DonwloadFormat::Csv => {
            let mut wrt = csv::Writer::from_writer(vec![]);
            let csv_data = data.to_csv();

            for d in csv_data {
                wrt.serialize(&d)?;
            }

            String::from_utf8(wrt.into_inner()?)?.into_bytes()
        }
        DonwloadFormat::Json => {
            let json = serde_json::to_string_pretty(&data)?;
            json.as_bytes().to_vec()
        }
    };

    let file = AttachmentType::Bytes {
        data: Cow::from(data_bytes),
        filename: format!("signups.{}", data.output),
    };

    let msg = msg
        .channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.title("Download");
                e.field(
                    "Details",
                    format!(
                        "Format: {}\nCreated: <t:{}>",
                        data.output,
                        data.created.timestamp()
                    ),
                    false,
                );
                e.field(
                    "Raids",
                    data.raids
                        .iter()
                        .map(|t| {
                            format!(
                                "\n__{}__\nId: {}\nData: <t:{}>",
                                t.raid.title,
                                t.raid.id,
                                t.raid.date.timestamp()
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("\n"),
                    false,
                );
                e.field(
                    "Log",
                    match log.is_empty() {
                        true => String::from("No errors"),
                        false => log
                            .iter()
                            .map(|s| format!("`{}`", s))
                            .collect::<Vec<_>>()
                            .join("\n"),
                    },
                    false,
                )
            });
            m.add_file(file)
        })
        .await?;

    aci.edit_quick_success(ctx, format!("[Done]({})", msg.link()))
        .await?;

    Ok(())
}

async fn info(
    ctx: &Context,
    aci: &ApplicationCommandInteraction,
    option: &ApplicationCommandInteractionDataOption,
    trace: LogTrace,
) -> Result<()> {
    let cmds = command_map(option);

    let id = cmds
        .get("id")
        .and_then(|v| v.as_i64())
        .context("Expected id field")
        .map_err_reply(|what| aci.create_quick_error(ctx, what, true))
        .await?;

    let public = cmds
        .get("public")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    trace.step("Loading raid information");

    let raid = db::Raid::by_id(ctx, id as i32)
        .await
        .with_context(|| format!("Failed to load raid with id: {}", id))
        .map_err_reply(|what| aci.create_quick_error(ctx, what, true))
        .await?;

    let bosses = raid.all_raid_bosses(ctx).await?;

    let roles = raid.all_roles(ctx).await?;

    // HashMap with Role id as key and value to keep count
    let mut roles_count = roles.iter().map(|r| (r.id, 0)).collect::<HashMap<_, _>>();

    trace.step("Loading signups to calculate role count");
    let signups = raid.get_signups(ctx).await?;

    future::try_join_all(signups.iter().map(|s| s.get_roles(ctx)))
        .await?
        .into_iter()
        .flatten()
        .for_each(|sr| {
            roles_count.entry(sr.id).and_modify(|e| *e += 1);
        });

    trace.step("Replying to user");
    aci.create_interaction_response(ctx, |r| {
        r.kind(InteractionResponseType::ChannelMessageWithSource);
        r.interaction_response_data(|d| {
            if !public {
                d.flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL);
            }
            let mut emb = CreateEmbed::xdefault();
            emb.field("Raid", raid.title, false);
            emb.field(
                "Date/Time",
                format!("<t:{}>", raid.date.timestamp()),
                false,
            );

            if !bosses.is_empty() {
                emb.fields_chunked_fmt(
                    &bosses,
                    |b| {
                        let boss_link = match &b.url {
                            Some(l) => format!("[{}]({})", b.name, l),
                            None => b.name.to_string(),
                        };
                        format!(
                            "{} | {}",
                            Mention::from(EmojiId::from(b.emoji as u64)),
                            boss_link
                        )
                    },
                    "Boss Pool",
                    false,
                    20,
                );
            }
            emb.fields_chunked_fmt(
                &roles,
                |r| {
                    format!(
                        "{} | {} : {}",
                        Mention::from(EmojiId::from(r.emoji as u64)),
                        r.title,
                        roles_count.get(&r.id).unwrap()
                    )
                },
                "Sign-up Count",
                true,
                20,
            );
            d.add_embed(emb)
        })
    })
    .await?;

    Ok(())
}

async fn list(
    ctx: &Context,
    aci: &ApplicationCommandInteraction,
    option: &ApplicationCommandInteractionDataOption,
    trace: LogTrace,
) -> Result<()> {
    let cmds = command_map(option);

    trace.step("Loading raid's");
    let mut raids = raids_from_days(ctx, cmds.get("day").unwrap().as_str().unwrap())
        .await
        .map_err_reply(|what| aci.create_quick_error(ctx, what, true))
        .await?;
    raids.sort_by_key(|t| t.date);

    let mut embeds: Vec<CreateEmbed> = Vec::new();
    let mut data_grouped = Vec::new();
    for (key, group) in &raids.into_iter().group_by(|a| a.date.date()) {
        data_grouped.push((key, group.collect::<Vec<_>>()));
    }
    for (d, ts) in data_grouped {
        let mut emb = CreateEmbed::xdefault();
        emb.title(d);
        for t in ts {
            emb.field(
                &t.title,
                format!("<t:{}>\nId: {}", t.date.timestamp(), t.id),
                true,
            );
        }
        embeds.push(emb);
    }

    trace.step("Replying");
    aci.create_interaction_response(ctx, |r| {
        r.kind(InteractionResponseType::ChannelMessageWithSource);
        r.interaction_response_data(|d| {
            d.flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL);
            d.embeds(embeds)
        })
    })
    .await?;
    Ok(())
}
