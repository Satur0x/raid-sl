use serenity::model::gateway::Activity;
use serenity::prelude::*;

use crate::{db, logging};

const DIZZY_EMOJI: char = '😵';
const GREEN_CIRCLE_EMOJI: char = '🟢';
const RED_CIRCLE_EMOJI: char = '🔴';

pub async fn update_status(ctx: &Context) {
    logging::log_discord_err_only(
        ctx,
        logging::LogInfo::automatic("Updating status"),
        |trace| async move {
            trace.step("Loading raid(s)");
            let raids_count = db::Raid::amount_by_state(ctx, db::RaidState::Open).await;
            let activity = match raids_count {
                Ok(0) => Activity::playing(format!("{} No raid available", RED_CIRCLE_EMOJI)),
                Ok(1) => Activity::playing(format!("{} 1 raid available", GREEN_CIRCLE_EMOJI,)),
                Ok(n) => {
                    Activity::playing(format!("{} {} raids available", GREEN_CIRCLE_EMOJI, n))
                }
                Err(_) => Activity::playing(format!("{} figuring out some issues", DIZZY_EMOJI)),
            };

            trace.step("Setting activity");
            ctx.set_activity(activity).await;
            Ok(())
        },
    )
    .await;
}
