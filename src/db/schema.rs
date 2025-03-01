#![allow(unused_imports)]

table! {
    use diesel::sql_types::*;
    use crate::db::*;

    /// Representation of the `config` table.
    ///
    /// (Automatically generated by Diesel.)
    config (name) {
        /// The `name` column of the `config` table.
        ///
        /// Its SQL type is `Text`.
        ///
        /// (Automatically generated by Diesel.)
        name -> Text,
        /// The `value` column of the `config` table.
        ///
        /// Its SQL type is `Text`.
        ///
        /// (Automatically generated by Diesel.)
        value -> Text,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::*;

    /// Representation of the `roles` table.
    ///
    /// (Automatically generated by Diesel.)
    roles (id) {
        /// The `id` column of the `roles` table.
        ///
        /// Its SQL type is `Int4`.
        ///
        /// (Automatically generated by Diesel.)
        id -> Int4,
        /// The `title` column of the `roles` table.
        ///
        /// Its SQL type is `Text`.
        ///
        /// (Automatically generated by Diesel.)
        title -> Text,
        /// The `repr` column of the `roles` table.
        ///
        /// Its SQL type is `Text`.
        ///
        /// (Automatically generated by Diesel.)
        repr -> Text,
        /// The `emoji` column of the `roles` table.
        ///
        /// Its SQL type is `Int8`.
        ///
        /// (Automatically generated by Diesel.)
        emoji -> Int8,
        /// The `active` column of the `roles` table.
        ///
        /// Its SQL type is `Bool`.
        ///
        /// (Automatically generated by Diesel.)
        active -> Bool,
        /// The `priority` column of the `roles` table.
        ///
        /// Its SQL type is `Int2`.
        ///
        /// (Automatically generated by Diesel.)
        priority -> Int2,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::*;

    /// Representation of the `signup_board_channels` table.
    ///
    /// (Automatically generated by Diesel.)
    signup_board_channels (day) {
        /// The `day` column of the `signup_board_channels` table.
        ///
        /// Its SQL type is `Date`.
        ///
        /// (Automatically generated by Diesel.)
        day -> Date,
        /// The `channel_id` column of the `signup_board_channels` table.
        ///
        /// Its SQL type is `Int8`.
        ///
        /// (Automatically generated by Diesel.)
        channel_id -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::*;

    /// Representation of the `signup_boss_preference_mappings` table.
    ///
    /// (Automatically generated by Diesel.)
    signup_boss_preference_mappings (signup_id, raid_boss_id) {
        /// The `signup_id` column of the `signup_boss_preference_mappings` table.
        ///
        /// Its SQL type is `Int4`.
        ///
        /// (Automatically generated by Diesel.)
        signup_id -> Int4,
        /// The `raid_boss_id` column of the `signup_boss_preference_mappings` table.
        ///
        /// Its SQL type is `Int4`.
        ///
        /// (Automatically generated by Diesel.)
        raid_boss_id -> Int4,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::*;

    /// Representation of the `signup_roles` table.
    ///
    /// (Automatically generated by Diesel.)
    signup_roles (signup_id, role_id) {
        /// The `signup_id` column of the `signup_roles` table.
        ///
        /// Its SQL type is `Int4`.
        ///
        /// (Automatically generated by Diesel.)
        signup_id -> Int4,
        /// The `role_id` column of the `signup_roles` table.
        ///
        /// Its SQL type is `Int4`.
        ///
        /// (Automatically generated by Diesel.)
        role_id -> Int4,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::*;

    /// Representation of the `signups` table.
    ///
    /// (Automatically generated by Diesel.)
    signups (id) {
        /// The `id` column of the `signups` table.
        ///
        /// Its SQL type is `Int4`.
        ///
        /// (Automatically generated by Diesel.)
        id -> Int4,
        /// The `user_id` column of the `signups` table.
        ///
        /// Its SQL type is `Int4`.
        ///
        /// (Automatically generated by Diesel.)
        user_id -> Int4,
        /// The `raid_id` column of the `signups` table.
        ///
        /// Its SQL type is `Int4`.
        ///
        /// (Automatically generated by Diesel.)
        raid_id -> Int4,
        /// The `comment` column of the `signups` table.
        ///
        /// Its SQL type is `Nullable<Text>`.
        ///
        /// (Automatically generated by Diesel.)
        comment -> Nullable<Text>,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::*;

    /// Representation of the `tier_mappings` table.
    ///
    /// (Automatically generated by Diesel.)
    tier_mappings (tier_id, discord_role_id) {
        /// The `tier_id` column of the `tier_mappings` table.
        ///
        /// Its SQL type is `Int4`.
        ///
        /// (Automatically generated by Diesel.)
        tier_id -> Int4,
        /// The `discord_role_id` column of the `tier_mappings` table.
        ///
        /// Its SQL type is `Int8`.
        ///
        /// (Automatically generated by Diesel.)
        discord_role_id -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::*;

    /// Representation of the `tiers` table.
    ///
    /// (Automatically generated by Diesel.)
    tiers (id) {
        /// The `id` column of the `tiers` table.
        ///
        /// Its SQL type is `Int4`.
        ///
        /// (Automatically generated by Diesel.)
        id -> Int4,
        /// The `name` column of the `tiers` table.
        ///
        /// Its SQL type is `Text`.
        ///
        /// (Automatically generated by Diesel.)
        name -> Text,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::*;

    /// Representation of the `raid_boss_mappings` table.
    ///
    /// (Automatically generated by Diesel.)
    raid_boss_mappings (raid_id, raid_boss_id) {
        /// The `raid_id` column of the `raid_boss_mappings` table.
        ///
        /// Its SQL type is `Int4`.
        ///
        /// (Automatically generated by Diesel.)
        raid_id -> Int4,
        /// The `raid_boss_id` column of the `raid_boss_mappings` table.
        ///
        /// Its SQL type is `Int4`.
        ///
        /// (Automatically generated by Diesel.)
        raid_boss_id -> Int4,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::*;

    /// Representation of the `raid_bosses` table.
    ///
    /// (Automatically generated by Diesel.)
    raid_bosses (id) {
        /// The `id` column of the `raid_bosses` table.
        ///
        /// Its SQL type is `Int4`.
        ///
        /// (Automatically generated by Diesel.)
        id -> Int4,
        /// The `repr` column of the `raid_bosses` table.
        ///
        /// Its SQL type is `Text`.
        ///
        /// (Automatically generated by Diesel.)
        repr -> Text,
        /// The `name` column of the `raid_bosses` table.
        ///
        /// Its SQL type is `Text`.
        ///
        /// (Automatically generated by Diesel.)
        name -> Text,
        /// The `wing` column of the `raid_bosses` table.
        ///
        /// Its SQL type is `Int4`.
        ///
        /// (Automatically generated by Diesel.)
        wing -> Int4,
        /// The `position` column of the `raid_bosses` table.
        ///
        /// Its SQL type is `Int4`.
        ///
        /// (Automatically generated by Diesel.)
        position -> Int4,
        /// The `emoji` column of the `raid_bosses` table.
        ///
        /// Its SQL type is `Int8`.
        ///
        /// (Automatically generated by Diesel.)
        emoji -> Int8,
        /// The `url` column of the `raid_bosses` table.
        ///
        /// Its SQL type is `Nullable<Text>`.
        ///
        /// (Automatically generated by Diesel.)
        url -> Nullable<Text>,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::*;

    /// Representation of the `raid_roles` table.
    ///
    /// (Automatically generated by Diesel.)
    raid_roles (raid_id, role_id) {
        /// The `raid_id` column of the `raid_roles` table.
        ///
        /// Its SQL type is `Int4`.
        ///
        /// (Automatically generated by Diesel.)
        raid_id -> Int4,
        /// The `role_id` column of the `raid_roles` table.
        ///
        /// Its SQL type is `Int4`.
        ///
        /// (Automatically generated by Diesel.)
        role_id -> Int4,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::*;

    /// Representation of the `raids` table.
    ///
    /// (Automatically generated by Diesel.)
    raids (id) {
        /// The `id` column of the `raids` table.
        ///
        /// Its SQL type is `Int4`.
        ///
        /// (Automatically generated by Diesel.)
        id -> Int4,
        /// The `title` column of the `raids` table.
        ///
        /// Its SQL type is `Text`.
        ///
        /// (Automatically generated by Diesel.)
        title -> Text,
        /// The `date` column of the `raids` table.
        ///
        /// Its SQL type is `Timestamp`.
        ///
        /// (Automatically generated by Diesel.)
        date -> Timestamp,
        /// The `state` column of the `raids` table.
        ///
        /// Its SQL type is `Raid_state`.
        ///
        /// (Automatically generated by Diesel.)
        state -> Raid_state,
        /// The `tier_id` column of the `raids` table.
        ///
        /// Its SQL type is `Nullable<Int4>`.
        ///
        /// (Automatically generated by Diesel.)
        tier_id -> Nullable<Int4>,
        /// The `board_message_id` column of the `raids` table.
        ///
        /// Its SQL type is `Nullable<Int8>`.
        ///
        /// (Automatically generated by Diesel.)
        board_message_id -> Nullable<Int8>,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::*;

    /// Representation of the `users` table.
    ///
    /// (Automatically generated by Diesel.)
    users (id) {
        /// The `id` column of the `users` table.
        ///
        /// Its SQL type is `Int4`.
        ///
        /// (Automatically generated by Diesel.)
        id -> Int4,
        /// The `discord_id` column of the `users` table.
        ///
        /// Its SQL type is `Int8`.
        ///
        /// (Automatically generated by Diesel.)
        discord_id -> Int8,
        /// The `gw2_id` column of the `users` table.
        ///
        /// Its SQL type is `Text`.
        ///
        /// (Automatically generated by Diesel.)
        gw2_id -> Text,
    }
}

joinable!(signup_boss_preference_mappings -> signups (signup_id));
joinable!(signup_boss_preference_mappings -> raid_bosses (raid_boss_id));
joinable!(signup_roles -> roles (role_id));
joinable!(signup_roles -> signups (signup_id));
joinable!(signups -> raids (raid_id));
joinable!(signups -> users (user_id));
joinable!(tier_mappings -> tiers (tier_id));
joinable!(raid_boss_mappings -> raid_bosses (raid_boss_id));
joinable!(raid_boss_mappings -> raids (raid_id));
joinable!(raid_roles -> roles (role_id));
joinable!(raid_roles -> raids (raid_id));
joinable!(raids -> tiers (tier_id));

allow_tables_to_appear_in_same_query!(
    config,
    roles,
    signup_board_channels,
    signup_boss_preference_mappings,
    signup_roles,
    signups,
    tier_mappings,
    tiers,
    raid_boss_mappings,
    raid_bosses,
    raid_roles,
    raids,
    users,
);
