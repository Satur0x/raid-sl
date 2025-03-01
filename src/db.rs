//! # db
//! This file contains abstractions for diesel sql query calls. A global connection pool
//! is used to hold connections and allowing diesel calls to be move to a blocking thread
//! with tokio task::spawn_blocking to not block on the executer thread

use crate::data::DBPoolData;
use chrono::{NaiveDate, NaiveDateTime};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::result::QueryResult;
use serenity::client::Context;
use serenity::model::{
    id::{EmojiId, MessageId, UserId},
    misc::Mention,
};
use std::env;
use std::sync::Arc;
use tokio::task;
use url::Url;

pub mod models;
pub mod schema;

pub use models::*;
use schema::*;

pub struct DBPool(Pool<ConnectionManager<PgConnection>>);

impl DBPool {
    pub fn new() -> Self {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        DBPool(Pool::new(manager).unwrap())
    }

    async fn load(ctx: &Context) -> Arc<Self> {
        ctx.data.read().await.get::<DBPoolData>().unwrap().clone()
    }

    fn conn(&self) -> PooledConnection<ConnectionManager<PgConnection>> {
        self.0.get().unwrap()
    }
}

impl Default for DBPool {
    fn default() -> Self {
        Self::new()
    }
}

// Insert und Upsert
async fn upsert_user(ctx: &Context, user: NewUser) -> QueryResult<User> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || {
        diesel::insert_into(users::table)
            .values(&user)
            .on_conflict(users::discord_id)
            .do_update()
            .set(&user)
            .get_result(&pool.conn())
    })
    .await
    .unwrap()
}

async fn insert_raid(ctx: &Context, t: NewRaid) -> QueryResult<Raid> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || {
        diesel::insert_into(raids::table)
            .values(&t)
            .get_result(&pool.conn())
    })
    .await
    .unwrap()
}

async fn insert_role(ctx: &Context, r: NewRole) -> QueryResult<Role> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || {
        diesel::insert_into(roles::table)
            .values(&r)
            .get_result(&pool.conn())
    })
    .await
    .unwrap()
}

async fn insert_tier(ctx: &Context, t: NewTier) -> QueryResult<Tier> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || {
        diesel::insert_into(tiers::table)
            .values(&t)
            .get_result(&pool.conn())
    })
    .await
    .unwrap()
}

async fn insert_raid_role(ctx: &Context, tr: NewRaidRole) -> QueryResult<RaidRole> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || {
        diesel::insert_into(raid_roles::table)
            .values(&tr)
            .get_result(&pool.conn())
    })
    .await
    .unwrap()
}

async fn insert_raid_boss_mapping(
    ctx: &Context,
    tbm: RaidBossMapping,
) -> QueryResult<RaidBossMapping> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || {
        diesel::insert_into(raid_boss_mappings::table)
            .values(&tbm)
            .get_result(&pool.conn())
    })
    .await
    .unwrap()
}

async fn insert_signup(ctx: &Context, s: NewSignup) -> QueryResult<Signup> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || {
        diesel::insert_into(signups::table)
            .values(&s)
            .get_result(&pool.conn())
    })
    .await
    .unwrap()
}

async fn insert_signup_role(ctx: &Context, sr: NewSignupRole) -> QueryResult<SignupRole> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || {
        diesel::insert_into(signup_roles::table)
            .values(&sr)
            .get_result(&pool.conn())
    })
    .await
    .unwrap()
}

async fn insert_tier_mapping(ctx: &Context, tm: NewTierMapping) -> QueryResult<TierMapping> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || {
        diesel::insert_into(tier_mappings::table)
            .values(&tm)
            .get_result(&pool.conn())
    })
    .await
    .unwrap()
}

async fn insert_raid_boss(ctx: &Context, tb: NewRaidBoss) -> QueryResult<RaidBoss> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || {
        diesel::insert_into(raid_bosses::table)
            .values(tb)
            .get_result(&pool.conn())
    })
    .await
    .unwrap()
}

async fn upsert_config(ctx: &Context, conf: Config) -> QueryResult<Config> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || {
        diesel::insert_into(config::table)
            .values(&conf)
            .on_conflict(config::name)
            .do_update()
            .set(config::value.eq(&conf.value))
            .get_result(&pool.conn())
    })
    .await
    .unwrap()
}

// Delete
async fn delete_user_by_id(ctx: &Context, id: i32) -> QueryResult<usize> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || diesel::delete(users::table.find(id)).execute(&pool.conn()))
        .await
        .unwrap()
}

async fn delete_signup_roles_by_signup(ctx: &Context, id: i32) -> QueryResult<usize> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || {
        diesel::delete(signup_roles::table.filter(signup_roles::signup_id.eq(id)))
            .execute(&pool.conn())
    })
    .await
    .unwrap()
}

async fn delete_signup_by_id(ctx: &Context, id: i32) -> QueryResult<usize> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || diesel::delete(signups::table.find(id)).execute(&pool.conn()))
        .await
        .unwrap()
}

async fn delete_tier_by_id(ctx: &Context, id: i32) -> QueryResult<usize> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || diesel::delete(tiers::table.find(id)).execute(&pool.conn()))
        .await
        .unwrap()
}

async fn delete_tier_mapping(
    ctx: &Context,
    tier_id: i32,
    discord_role_id: i64,
) -> QueryResult<usize> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || {
        diesel::delete(tier_mappings::table.find((tier_id, discord_role_id))).execute(&pool.conn())
    })
    .await
    .unwrap()
}

async fn delete_raid_boss_by_id(ctx: &Context, id: i32) -> QueryResult<usize> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || {
        diesel::delete(raid_bosses::table.find(id)).execute(&pool.conn())
    })
    .await
    .unwrap()
}

// Select
async fn select_user_by_id(ctx: &Context, id: i32) -> QueryResult<User> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || users::table.find(id).first(&pool.conn()))
        .await
        .unwrap()
}

async fn select_user_by_discord_id(ctx: &Context, discord_id: u64) -> QueryResult<User> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || {
        users::table
            .filter(users::discord_id.eq(discord_id as i64))
            .first(&pool.conn())
    })
    .await
    .unwrap()
}

async fn select_users_with_signup_by_date(
    ctx: &Context,
    date: NaiveDate,
) -> QueryResult<Vec<User>> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || {
        let join = users::table.inner_join(signups::table.inner_join(raids::table));
        join.filter(raids::date.ge(date.and_hms(0, 0, 0)))
            .filter(raids::date.le(date.and_hms(23, 59, 59)))
            .select(users::all_columns)
            .load(&pool.conn())
    })
    .await
    .unwrap()
}

async fn select_all_signups_by_user(ctx: &Context, user_id: i32) -> QueryResult<Vec<Signup>> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || {
        let join = signups::table
            .inner_join(users::table)
            .inner_join(raids::table);
        join.filter(users::id.eq(user_id))
            .select(signups::all_columns)
            .load(&pool.conn())
    })
    .await
    .unwrap()
}

async fn select_joined_active_raids_by_user(
    ctx: &Context,
    user_id: i32,
) -> QueryResult<Vec<Raid>> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || {
        let join = signups::table
            .inner_join(users::table)
            .inner_join(raids::table);
        join.filter(users::id.eq(user_id))
            .filter(
                raids::state
                    .eq(RaidState::Open)
                    .or(raids::state.eq(RaidState::Closed))
                    .or(raids::state.eq(RaidState::Started)),
            )
            .select(raids::all_columns)
            .load(&pool.conn())
    })
    .await
    .unwrap()
}

async fn select_active_signups_raids_by_user(
    ctx: &Context,
    user_id: i32,
) -> QueryResult<Vec<(Signup, Raid)>> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || {
        let join = signups::table
            .inner_join(users::table)
            .inner_join(raids::table);
        join.filter(users::id.eq(user_id))
            .filter(
                raids::state
                    .eq(RaidState::Open)
                    .or(raids::state.eq(RaidState::Closed))
                    .or(raids::state.eq(RaidState::Started)),
            )
            .select((signups::all_columns, raids::all_columns))
            .load(&pool.conn())
    })
    .await
    .unwrap()
}

async fn select_active_signups_by_user(ctx: &Context, user_id: i32) -> QueryResult<Vec<Signup>> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || {
        let join = signups::table
            .inner_join(users::table)
            .inner_join(raids::table);
        join.filter(users::id.eq(user_id))
            .filter(
                raids::state
                    .eq(RaidState::Open)
                    .or(raids::state.eq(RaidState::Closed))
                    .or(raids::state.eq(RaidState::Started)),
            )
            .select(signups::all_columns)
            .load(&pool.conn())
    })
    .await
    .unwrap()
}

async fn select_open_signups_by_user(ctx: &Context, user_id: i32) -> QueryResult<Vec<Signup>> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || {
        let join = signups::table
            .inner_join(users::table)
            .inner_join(raids::table);
        join.filter(users::id.eq(user_id))
            .filter(raids::state.eq(RaidState::Open))
            .select(signups::all_columns)
            .load(&pool.conn())
    })
    .await
    .unwrap()
}

async fn select_raid_by_id(ctx: &Context, id: i32) -> QueryResult<Raid> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || raids::table.find(id).first(&pool.conn()))
        .await
        .unwrap()
}

async fn select_raids_by_state(
    ctx: &Context,
    state: RaidState,
) -> QueryResult<Vec<Raid>> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || {
        raids::table
            .filter(raids::state.eq(state))
            .load(&pool.conn())
    })
    .await
    .unwrap()
}

async fn select_raid_by_id_and_state(
    ctx: &Context,
    id: i32,
    state: RaidState,
) -> QueryResult<Raid> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || {
        raids::table
            .find(id)
            .filter(raids::state.eq(state))
            .first::<Raid>(&pool.conn())
    })
    .await
    .unwrap()
}

async fn select_active_raids(ctx: &Context) -> QueryResult<Vec<Raid>> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || {
        raids::table
            .filter(
                raids::state
                    .eq(RaidState::Open)
                    .or(raids::state.eq(RaidState::Closed))
                    .or(raids::state.eq(RaidState::Started)),
            )
            .load::<Raid>(&pool.conn())
    })
    .await
    .unwrap()
}

async fn select_raids_by_tier(ctx: &Context, id: i32) -> QueryResult<Vec<Raid>> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || {
        let join = raids::table.inner_join(tiers::table);
        join.filter(tiers::id.eq(id))
            .select(raids::all_columns)
            .load(&pool.conn())
    })
    .await
    .unwrap()
}

async fn select_raids_by_date(ctx: &Context, date: NaiveDate) -> QueryResult<Vec<Raid>> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || {
        raids::table
            .filter(raids::date.ge(date.and_hms(0, 0, 0)))
            .filter(raids::date.le(date.and_hms(23, 59, 59)))
            .select(raids::all_columns)
            .load(&pool.conn())
    })
    .await
    .unwrap()
}

async fn select_signups_by_raid(ctx: &Context, id: i32) -> QueryResult<Vec<Signup>> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || {
        let join = signups::table.inner_join(raids::table);
        join.filter(raids::id.eq(id))
            .select(signups::all_columns)
            .load(&pool.conn())
    })
    .await
    .unwrap()
}

async fn select_signups_by_date(ctx: &Context, date: NaiveDate) -> QueryResult<Vec<Signup>> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || {
        let join = signups::table.inner_join(raids::table);
        join.filter(raids::date.ge(date.and_hms(0, 0, 0)))
            .filter(raids::date.le(date.and_hms(23, 59, 59)))
            .select(signups::all_columns)
            .load(&pool.conn())
    })
    .await
    .unwrap()
}

async fn select_signup_by_user_and_raid(
    ctx: &Context,
    user_id: i32,
    raid_id: i32,
) -> QueryResult<Signup> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || {
        signups::table
            .filter(signups::user_id.eq(user_id))
            .filter(signups::raid_id.eq(raid_id))
            .first(&pool.conn())
    })
    .await
    .unwrap()
}

async fn select_signup_by_discord_user_and_raid(
    ctx: &Context,
    discord_id: i64,
    raid_id: i32,
) -> QueryResult<Signup> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || {
        let join = signups::table.inner_join(users::table);
        join.filter(users::discord_id.eq(discord_id))
            .filter(signups::raid_id.eq(raid_id))
            .select(signups::all_columns)
            .first(&pool.conn())
    })
    .await
    .unwrap()
}

async fn select_all_tiers(ctx: &Context) -> QueryResult<Vec<Tier>> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || tiers::table.load(&pool.conn()))
        .await
        .unwrap()
}

async fn select_tier_by_id(ctx: &Context, id: i32) -> QueryResult<Tier> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || tiers::table.find(id).first(&pool.conn()))
        .await
        .unwrap()
}

async fn select_tier_by_name(ctx: &Context, name: String) -> QueryResult<Tier> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || {
        tiers::table
            .filter(tiers::name.eq(name))
            .first(&pool.conn())
    })
    .await
    .unwrap()
}

async fn select_tier_mappings_by_tier(ctx: &Context, id: i32) -> QueryResult<Vec<TierMapping>> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || {
        let join = tier_mappings::table.inner_join(tiers::table);
        join.filter(tiers::id.eq(id))
            .select(tier_mappings::all_columns)
            .load(&pool.conn())
    })
    .await
    .unwrap()
}

async fn select_tier_mappings_by_tier_and_discord_role(
    ctx: &Context,
    tier_id: i32,
    discord_id: i64,
) -> QueryResult<TierMapping> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || {
        let join = tier_mappings::table.inner_join(tiers::table);
        join.filter(tiers::id.eq(tier_id))
            .filter(tier_mappings::discord_role_id.eq(discord_id))
            .select(tier_mappings::all_columns)
            .first(&pool.conn())
    })
    .await
    .unwrap()
}

async fn select_raid_roles_by_raid(
    ctx: &Context,
    id: i32,
) -> QueryResult<Vec<RaidRole>> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || {
        let join = raid_roles::table.inner_join(raids::table);
        join.filter(raids::id.eq(id))
            .select(raid_roles::all_columns)
            .load(&pool.conn())
    })
    .await
    .unwrap()
}

async fn select_roles_by_active(ctx: &Context, active: bool) -> QueryResult<Vec<Role>> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || {
        roles::table
            .filter(roles::active.eq(active))
            .order_by(roles::priority.desc())
            .then_order_by(roles::title)
            .get_results(&pool.conn())
    })
    .await
    .unwrap()
}

async fn select_active_role_by_emoji(ctx: &Context, emoji_id: i64) -> QueryResult<Role> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || {
        roles::table
            .filter(roles::active.eq(true))
            .filter(roles::emoji.eq(emoji_id))
            .first(&pool.conn())
    })
    .await
    .unwrap()
}

async fn select_active_role_by_repr(ctx: &Context, repr: String) -> QueryResult<Role> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || {
        roles::table
            .filter(roles::active.eq(true))
            .filter(roles::repr.eq(repr))
            .first(&pool.conn())
    })
    .await
    .unwrap()
}

async fn select_roles_by_raid(ctx: &Context, id: i32) -> QueryResult<Vec<Role>> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || {
        let join = raid_roles::table
            .inner_join(raids::table)
            .inner_join(roles::table);
        join.filter(raids::id.eq(id))
            .select(roles::all_columns)
            .order_by(roles::priority.desc())
            .then_order_by(roles::title)
            .load(&pool.conn())
    })
    .await
    .unwrap()
}

async fn select_active_roles_by_raid(ctx: &Context, id: i32) -> QueryResult<Vec<Role>> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || {
        let join = raid_roles::table
            .inner_join(raids::table)
            .inner_join(roles::table);
        join.filter(raids::id.eq(id))
            .filter(roles::active.eq(true))
            .select(roles::all_columns)
            .order_by(roles::priority.desc())
            .then_order_by(roles::title)
            .load(&pool.conn())
    })
    .await
    .unwrap()
}

async fn select_roles_by_signup(ctx: &Context, id: i32) -> QueryResult<Vec<Role>> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || {
        let join = signup_roles::table
            .inner_join(signups::table)
            .inner_join(roles::table);
        join.filter(signups::id.eq(id))
            .select(roles::all_columns)
            .order_by(roles::priority.desc())
            .then_order_by(roles::title)
            .load(&pool.conn())
    })
    .await
    .unwrap()
}

async fn select_config_by_name(ctx: &Context, name: String) -> QueryResult<Config> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || config::table.find(name).first(&pool.conn()))
        .await
        .unwrap()
}

async fn select_all_raid_bosses(ctx: &Context) -> QueryResult<Vec<RaidBoss>> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || raid_bosses::table.load(&pool.conn()))
        .await
        .unwrap()
}

async fn select_raid_boss_by_repr(ctx: &Context, repr: String) -> QueryResult<RaidBoss> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || {
        raid_bosses::table
            .filter(raid_bosses::repr.eq(repr))
            .first(&pool.conn())
    })
    .await
    .unwrap()
}

async fn select_raid_bosses_by_raid(
    ctx: &Context,
    id: i32,
) -> QueryResult<Vec<RaidBoss>> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || {
        raid_boss_mappings::table
            .inner_join(raids::table)
            .inner_join(raid_bosses::table)
            .filter(raids::id.eq(id))
            .select(raid_bosses::all_columns)
            .load(&pool.conn())
    })
    .await
    .unwrap()
}

// Count
async fn count_raids_by_state(ctx: &Context, state: RaidState) -> QueryResult<i64> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || {
        raids::table
            .filter(raids::state.eq(state))
            .count()
            .get_result(&pool.conn())
    })
    .await
    .unwrap()
}

async fn count_signups_by_raid(ctx: &Context, raid_id: i32) -> QueryResult<i64> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || {
        signups::table
            .filter(signups::raid_id.eq(raid_id))
            .count()
            .get_result(&pool.conn())
    })
    .await
    .unwrap()
}

async fn count_active_raids_by_date(ctx: &Context, date: NaiveDate) -> QueryResult<i64> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || {
        raids::table
            .filter(raids::date.ge(date.and_hms(0, 0, 0)))
            .filter(raids::date.le(date.and_hms(23, 59, 59)))
            .filter(
                raids::state
                    .eq(RaidState::Open)
                    .or(raids::state.eq(RaidState::Closed))
                    .or(raids::state.eq(RaidState::Started)),
            )
            .count()
            .get_result(&pool.conn())
    })
    .await
    .unwrap()
}

// Update
async fn update_raid_state(
    ctx: &Context,
    id: i32,
    state: RaidState,
) -> QueryResult<Raid> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || {
        diesel::update(raids::table.find(id))
            .set(raids::state.eq(state))
            .get_result(&pool.conn())
    })
    .await
    .unwrap()
}

async fn update_raid_tier(
    ctx: &Context,
    id: i32,
    tier_id: Option<i32>,
) -> QueryResult<Raid> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || {
        diesel::update(raids::table.find(id))
            .set(raids::tier_id.eq(tier_id))
            .get_result(&pool.conn())
    })
    .await
    .unwrap()
}

async fn update_raid_board_message(
    ctx: &Context,
    id: i32,
    msg_id: Option<i64>,
) -> QueryResult<Raid> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || {
        diesel::update(raids::table.find(id))
            .set(raids::board_message_id.eq(msg_id))
            .get_result(&pool.conn())
    })
    .await
    .unwrap()
}

async fn update_role_active(ctx: &Context, id: i32, active: bool) -> QueryResult<Role> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || {
        diesel::update(roles::table.find(id))
            .set(roles::active.eq(active))
            .get_result(&pool.conn())
    })
    .await
    .unwrap()
}

async fn update_signup_comment(
    ctx: &Context,
    id: i32,
    comment: Option<String>,
) -> QueryResult<Signup> {
    let pool = DBPool::load(ctx).await;
    task::spawn_blocking(move || {
        diesel::update(signups::table.find(id))
            .set(signups::comment.eq(comment))
            .get_result(&pool.conn())
    })
    .await
    .unwrap()
}

/* --- User --- */
impl User {
    pub async fn upsert(ctx: &Context, discord_id: u64, gw2_id: String) -> QueryResult<User> {
        let user = NewUser {
            discord_id: discord_id as i64,
            gw2_id,
        };
        upsert_user(ctx, user).await
    }

    pub async fn delete(&self, ctx: &Context) -> QueryResult<usize> {
        delete_user_by_id(ctx, self.id).await
    }

    pub async fn by_discord_id(ctx: &Context, id: UserId) -> QueryResult<User> {
        select_user_by_discord_id(ctx, *id.as_u64()).await
    }

    pub async fn joined_active_raids(&self, ctx: &Context) -> QueryResult<Vec<Raid>> {
        select_joined_active_raids_by_user(ctx, self.id).await
    }

    pub async fn active_signups_with_raid(
        &self,
        ctx: &Context,
    ) -> QueryResult<Vec<(Signup, Raid)>> {
        select_active_signups_raids_by_user(ctx, self.id).await
    }

    pub async fn active_signups(&self, ctx: &Context) -> QueryResult<Vec<Signup>> {
        select_active_signups_by_user(ctx, self.id).await
    }

    pub async fn open_signups(&self, ctx: &Context) -> QueryResult<Vec<Signup>> {
        select_open_signups_by_user(ctx, self.id).await
    }

    pub async fn all_signups(&self, ctx: &Context) -> QueryResult<Vec<Signup>> {
        select_all_signups_by_user(ctx, self.id).await
    }

    pub async fn by_signed_up_and_date(ctx: &Context, date: NaiveDate) -> QueryResult<Vec<User>> {
        select_users_with_signup_by_date(ctx, date).await
    }
}

/* -- Raid -- */
impl Raid {
    pub async fn insert(
        ctx: &Context,
        title: String,
        date: NaiveDateTime,
        tier_id: Option<i32>,
    ) -> QueryResult<Raid> {
        let t = NewRaid {
            title,
            date,
            tier_id,
        };
        insert_raid(ctx, t).await
    }

    pub async fn by_state(ctx: &Context, state: RaidState) -> QueryResult<Vec<Raid>> {
        select_raids_by_state(ctx, state).await
    }

    pub async fn all_active(ctx: &Context) -> QueryResult<Vec<Raid>> {
        select_active_raids(ctx).await
    }

    pub async fn amount_by_state(ctx: &Context, state: RaidState) -> QueryResult<i64> {
        count_raids_by_state(ctx, state).await
    }

    pub async fn amount_active_by_day(ctx: &Context, date: NaiveDate) -> QueryResult<i64> {
        count_active_raids_by_date(ctx, date).await
    }

    pub async fn get_signup_count(&self, ctx: &Context) -> QueryResult<i64> {
        count_signups_by_raid(ctx, self.id).await
    }

    pub async fn by_id(ctx: &Context, id: i32) -> QueryResult<Raid> {
        select_raid_by_id(ctx, id).await
    }

    pub async fn by_id_and_state(
        ctx: &Context,
        id: i32,
        state: RaidState,
    ) -> QueryResult<Raid> {
        select_raid_by_id_and_state(ctx, id, state).await
    }

    pub async fn by_date(ctx: &Context, date: NaiveDate) -> QueryResult<Vec<Raid>> {
        select_raids_by_date(ctx, date).await
    }

    pub async fn set_state(self, ctx: &Context, state: RaidState) -> QueryResult<Raid> {
        update_raid_state(ctx, self.id, state).await
    }

    pub async fn get_tier(&self, ctx: &Context) -> Option<QueryResult<Tier>> {
        match self.tier_id {
            None => None,
            Some(id) => Some(select_tier_by_id(ctx, id).await),
        }
    }

    pub async fn set_tier(&self, ctx: &Context, tier_id: Option<i32>) -> QueryResult<Raid> {
        update_raid_tier(ctx, self.id, tier_id).await
    }

    pub async fn get_signups(&self, ctx: &Context) -> QueryResult<Vec<Signup>> {
        select_signups_by_raid(ctx, self.id).await
    }

    pub async fn add_role(&self, ctx: &Context, role_id: i32) -> QueryResult<RaidRole> {
        let raid_role = NewRaidRole {
            raid_id: self.id,
            role_id,
        };
        insert_raid_role(ctx, raid_role).await
    }

    pub async fn add_raid_boss(
        &self,
        ctx: &Context,
        raid_boss_id: i32,
    ) -> QueryResult<RaidBossMapping> {
        let mapping = RaidBossMapping {
            raid_id: self.id,
            raid_boss_id,
        };

        insert_raid_boss_mapping(ctx, mapping).await
    }

    pub async fn get_raid_roles(&self, ctx: &Context) -> QueryResult<Vec<RaidRole>> {
        select_raid_roles_by_raid(ctx, self.id).await
    }

    pub async fn all_roles(&self, ctx: &Context) -> QueryResult<Vec<Role>> {
        select_roles_by_raid(ctx, self.id).await
    }

    pub async fn all_raid_bosses(&self, ctx: &Context) -> QueryResult<Vec<RaidBoss>> {
        select_raid_bosses_by_raid(ctx, self.id).await
    }

    pub async fn active_roles(&self, ctx: &Context) -> QueryResult<Vec<Role>> {
        select_active_roles_by_raid(ctx, self.id).await
    }

    pub async fn set_board_msg(&self, ctx: &Context, msg_id: Option<u64>) -> QueryResult<Raid> {
        update_raid_board_message(ctx, self.id, msg_id.map(|id| id as i64)).await
    }

    pub fn board_message(&self) -> Option<MessageId> {
        self.board_message_id.map(|id| MessageId::from(id as u64))
    }
}

/* -- Signup -- */
impl Signup {
    pub async fn insert(ctx: &Context, user: &User, raid: &Raid) -> QueryResult<Self> {
        let new_signup = NewSignup {
            user_id: user.id,
            raid_id: raid.id,
        };
        insert_signup(ctx, new_signup).await
    }

    pub async fn add_role(&self, ctx: &Context, role: &Role) -> QueryResult<SignupRole> {
        let sr = NewSignupRole {
            signup_id: self.id,
            role_id: role.id,
        };
        insert_signup_role(ctx, sr).await
    }

    pub async fn update_comment(
        &self,
        ctx: &Context,
        comment: Option<String>,
    ) -> QueryResult<Self> {
        update_signup_comment(ctx, self.id, comment).await
    }

    pub async fn get_raid(&self, ctx: &Context) -> QueryResult<Raid> {
        select_raid_by_id(ctx, self.raid_id).await
    }

    pub async fn get_user(&self, ctx: &Context) -> QueryResult<User> {
        select_user_by_id(ctx, self.user_id).await
    }

    pub async fn get_roles(&self, ctx: &Context) -> QueryResult<Vec<Role>> {
        select_roles_by_signup(ctx, self.id).await
    }

    pub async fn clear_roles(&self, ctx: &Context) -> QueryResult<usize> {
        delete_signup_roles_by_signup(ctx, self.id).await
    }

    pub async fn by_user_and_raid(
        ctx: &Context,
        u: &User,
        t: &Raid,
    ) -> QueryResult<Signup> {
        select_signup_by_user_and_raid(ctx, u.id, t.id).await
    }

    pub async fn by_discord_user_and_raid(
        ctx: &Context,
        u: &UserId,
        t: &Raid,
    ) -> QueryResult<Signup> {
        select_signup_by_discord_user_and_raid(ctx, *u.as_u64() as i64, t.id).await
    }

    pub async fn by_date(ctx: &Context, date: NaiveDate) -> QueryResult<Vec<Signup>> {
        select_signups_by_date(ctx, date).await
    }

    pub async fn remove(self, ctx: &Context) -> QueryResult<usize> {
        delete_signup_by_id(ctx, self.id).await
    }
}

/* -- Role -- */

impl Role {
    pub async fn insert(
        ctx: &Context,
        title: String,
        repr: String,
        emoji: u64,
        priority: Option<i16>,
    ) -> QueryResult<Role> {
        let r = NewRole {
            title,
            repr,
            emoji: emoji as i64,
            priority,
        };
        insert_role(ctx, r).await
    }

    /// Deactivates the role but keeps it in database
    pub async fn deactivate(self, ctx: &Context) -> QueryResult<Role> {
        update_role_active(ctx, self.id, false).await
    }

    /// Loads all active roles
    pub async fn all_active(ctx: &Context) -> QueryResult<Vec<Role>> {
        select_roles_by_active(ctx, true).await
    }

    /// Loads the current active role associated with provided emoji
    pub async fn by_emoji(ctx: &Context, emoji: u64) -> QueryResult<Role> {
        select_active_role_by_emoji(ctx, emoji as i64).await
    }

    /// Loads the current active role with specified repr
    pub async fn by_repr(ctx: &Context, repr: String) -> QueryResult<Role> {
        select_active_role_by_repr(ctx, repr).await
    }
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} | {}",
            Mention::from(EmojiId::from(self.emoji as u64)),
            self.title
        )
    }
}

// --- Tier ---
impl Tier {
    pub async fn insert(ctx: &Context, name: String) -> QueryResult<Tier> {
        let new_tier = NewTier { name };
        insert_tier(ctx, new_tier).await
    }

    pub async fn all(ctx: &Context) -> QueryResult<Vec<Tier>> {
        select_all_tiers(ctx).await
    }

    pub async fn by_name(ctx: &Context, name: String) -> QueryResult<Tier> {
        select_tier_by_name(ctx, name).await
    }

    pub async fn add_discord_role(
        &self,
        ctx: &Context,
        discord_id: u64,
    ) -> QueryResult<TierMapping> {
        let new_tier_mapping = NewTierMapping {
            tier_id: self.id,
            discord_role_id: discord_id as i64,
        };
        insert_tier_mapping(ctx, new_tier_mapping).await
    }

    pub async fn delete(self, ctx: &Context) -> QueryResult<usize> {
        delete_tier_by_id(ctx, self.id).await
    }

    pub async fn get_discord_roles(&self, ctx: &Context) -> QueryResult<Vec<TierMapping>> {
        select_tier_mappings_by_tier(ctx, self.id).await
    }

    pub async fn get_tier_mapping_by_discord_role(
        &self,
        ctx: &Context,
        role_id: u64,
    ) -> QueryResult<TierMapping> {
        select_tier_mappings_by_tier_and_discord_role(ctx, self.id, role_id as i64).await
    }

    pub async fn get_raids(&self, ctx: &Context) -> QueryResult<Vec<Raid>> {
        select_raids_by_tier(ctx, self.id).await
    }
}

// --- TierMapping ---
impl TierMapping {
    pub async fn delete(self, ctx: &Context) -> QueryResult<usize> {
        delete_tier_mapping(ctx, self.tier_id, self.discord_role_id).await
    }
}

// --- Config ---
impl Config {
    pub async fn load(ctx: &Context, name: String) -> QueryResult<Config> {
        select_config_by_name(ctx, name).await
    }

    pub async fn save(self, ctx: &Context) -> QueryResult<Config> {
        upsert_config(ctx, self).await
    }
}

impl RaidBoss {
    pub async fn insert(
        ctx: &Context,
        name: String,
        repr: String,
        wing: i32,
        position: i32,
        emoji: EmojiId,
        url: Option<Url>,
    ) -> QueryResult<Self> {
        let tb = NewRaidBoss {
            name,
            repr,
            wing,
            position,
            emoji: emoji.0 as i64,
            url: url.map(|u| u.to_string()),
        };

        insert_raid_boss(ctx, tb).await
    }

    pub async fn all(ctx: &Context) -> QueryResult<Vec<Self>> {
        select_all_raid_bosses(ctx).await
    }

    pub async fn by_repr(ctx: &Context, repr: String) -> QueryResult<Self> {
        select_raid_boss_by_repr(ctx, repr).await
    }

    pub async fn delete(&self, ctx: &Context) -> QueryResult<usize> {
        delete_raid_boss_by_id(ctx, self.id).await
    }
}

impl std::fmt::Display for RaidBoss {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(url) = &self.url {
            write!(
                f,
                "{} | {} | [{}]({})",
                Mention::from(EmojiId::from(self.emoji as u64)),
                self.repr,
                self.name,
                url
            )
        } else {
            write!(
                f,
                "{} | {} | {}",
                Mention::from(EmojiId::from(self.emoji as u64)),
                self.repr,
                self.name
            )
        }
    }
}
