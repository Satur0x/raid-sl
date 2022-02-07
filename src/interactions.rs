use crate::{components::*, logging::*};

use serenity::{model::interactions::message_component::MessageComponentInteraction, prelude::*};

mod list_signups;
mod register_info;
mod select_training;

async fn button_general_interaction(
    ctx: &Context,
    mci: &MessageComponentInteraction,
    ovi: &OverviewMessageInteraction,
) -> () {
    log_discord(ctx, mci, |trace| async move {
        match ovi {
            OverviewMessageInteraction::List => list_signups::interaction(ctx, mci, trace).await,
            OverviewMessageInteraction::Register => {
                register_info::interaction(ctx, mci, trace).await
            }
            OverviewMessageInteraction::TrainingSelect => {
                select_training::interaction(ctx, mci, trace).await
            }
        }
    })
    .await
}

pub async fn button_interaction(ctx: &Context, mci: &MessageComponentInteraction) {
    // Check what interaction to handle
    if let Ok(bi) = mci.data.custom_id.parse::<GlobalInteraction>() {
        match &bi {
            GlobalInteraction::Overview(bgi) => button_general_interaction(ctx, mci, bgi).await,
        }
    };
}
