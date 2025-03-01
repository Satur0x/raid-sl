use crate::db::schema::{
    config, roles, signup_roles, signups, tier_mappings, tiers, raid_boss_mappings,
    raid_bosses, raid_roles, raids, users,
};
use diesel_derive_enum::DbEnum;
use serde::Serialize;
use std::{fmt, str};

use chrono::naive::NaiveDateTime;

#[derive(Identifiable, Queryable, PartialEq, Debug, Serialize)]
#[table_name = "users"]
pub struct User {
    pub id: i32,
    pub discord_id: i64,
    pub gw2_id: String,
}

impl User {
    pub fn discord_id(&self) -> u64 {
        self.discord_id as u64
    }
}

#[derive(Insertable, AsChangeset, Debug)]
#[table_name = "users"]
pub(super) struct NewUser {
    pub discord_id: i64,
    pub gw2_id: String,
}

#[derive(Identifiable, Queryable, Associations, Clone, PartialEq, Debug)]
#[belongs_to(User)]
#[belongs_to(Raid)]
#[table_name = "signups"]
pub struct Signup {
    pub id: i32,
    pub user_id: i32,
    pub raid_id: i32,
    pub comment: Option<String>,
}

#[derive(Insertable, Debug)]
#[table_name = "signups"]
pub struct NewSignup {
    pub user_id: i32,
    pub raid_id: i32,
}

#[derive(Debug, DbEnum, PartialEq, PartialOrd, Clone, Serialize)]
#[DieselType = "Raid_state"]
pub enum RaidState {
    Created,
    Open,
    Closed,
    Started,
    Finished,
}

impl fmt::Display for RaidState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RaidState::Created => write!(f, "created"),
            RaidState::Open => write!(f, "open"),
            RaidState::Closed => write!(f, "closed"),
            RaidState::Started => write!(f, "started"),
            RaidState::Finished => write!(f, "finished"),
        }
    }
}

impl str::FromStr for RaidState {
    type Err = String;

    fn from_str(input: &str) -> Result<RaidState, Self::Err> {
        match input {
            "created" => Ok(RaidState::Created),
            "open" => Ok(RaidState::Open),
            "closed" => Ok(RaidState::Closed),
            "started" => Ok(RaidState::Started),
            "finished" => Ok(RaidState::Finished),
            e => Err(format!("unknown raid state: {}", e)),
        }
    }
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug, Serialize, Clone)]
#[belongs_to(Tier)]
#[table_name = "raids"]
pub struct Raid {
    pub id: i32,
    pub title: String,
    pub date: NaiveDateTime,
    pub state: RaidState,
    pub tier_id: Option<i32>,
    pub board_message_id: Option<i64>,
}

#[derive(Insertable, Debug)]
#[table_name = "raids"]
pub(super) struct NewRaid {
    pub title: String,
    pub date: NaiveDateTime,
    pub tier_id: Option<i32>,
}

#[derive(Identifiable, Queryable, Associations, Hash, PartialEq, Eq, Debug, Serialize)]
#[table_name = "roles"]
pub struct Role {
    pub id: i32,
    pub title: String,
    pub repr: String,
    pub emoji: i64,
    pub active: bool,
    pub priority: i16,
}

#[derive(Insertable, Debug)]
#[table_name = "roles"]
pub(super) struct NewRole {
    pub title: String,
    pub repr: String,
    pub emoji: i64,
    pub priority: Option<i16>,
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(Signup)]
#[belongs_to(Role)]
#[table_name = "signup_roles"]
#[primary_key(signup_id, role_id)]
pub struct SignupRole {
    pub signup_id: i32,
    pub role_id: i32,
}

#[derive(Insertable, Debug)]
#[table_name = "signup_roles"]
pub(super) struct NewSignupRole {
    pub signup_id: i32,
    pub role_id: i32,
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(Raid)]
#[belongs_to(Role)]
#[table_name = "raid_roles"]
#[primary_key(raid_id, role_id)]
pub struct RaidRole {
    pub raid_id: i32,
    pub role_id: i32,
}

#[derive(Insertable, Debug)]
#[table_name = "raid_roles"]
pub(super) struct NewRaidRole {
    pub raid_id: i32,
    pub role_id: i32,
}

#[derive(Identifiable, Queryable, PartialEq, Debug)]
#[table_name = "tiers"]
pub struct Tier {
    pub id: i32,
    pub name: String,
}

#[derive(Insertable, Debug)]
#[table_name = "tiers"]
pub(super) struct NewTier {
    pub name: String,
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[table_name = "tier_mappings"]
#[belongs_to(Tier)]
#[primary_key(tier_id, discord_role_id)]
pub struct TierMapping {
    pub tier_id: i32,
    pub discord_role_id: i64,
}

#[derive(Insertable, Debug)]
#[table_name = "tier_mappings"]
pub(super) struct NewTierMapping {
    pub tier_id: i32,
    pub discord_role_id: i64,
}

#[derive(Queryable, Insertable, Debug)]
#[table_name = "config"]
pub struct Config {
    pub name: String,
    pub value: String,
}

#[derive(Identifiable, Queryable, Associations, Hash, PartialEq, Eq, Debug, Serialize)]
#[table_name = "raid_bosses"]
pub struct RaidBoss {
    pub id: i32,
    pub repr: String,
    pub name: String,
    pub wing: i32,
    pub position: i32,
    pub emoji: i64,
    pub url: Option<String>,
}

#[derive(Insertable, Associations, Debug)]
#[table_name = "raid_bosses"]
pub struct NewRaidBoss {
    pub repr: String,
    pub name: String,
    pub wing: i32,
    pub position: i32,
    pub emoji: i64,
    pub url: Option<String>,
}

#[derive(Insertable, Queryable, Associations, Debug, Hash, PartialEq, Eq)]
#[table_name = "raid_boss_mappings"]
pub struct RaidBossMapping {
    pub raid_id: i32,
    pub raid_boss_id: i32,
}
