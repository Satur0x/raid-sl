-- Your SQL goes here
CREATE TABLE raid_bosses (
	id SERIAL PRIMARY KEY,
	repr TEXT UNIQUE NOT NULL,
	name TEXT NOT NULL,
	wing INT NOT NULL,
	position INT NOT NULL,
	emoji BIGINT NOT NULL,
	url TEXT,
	UNIQUE (wing, position)
);
CREATE TABLE raid_boss_mappings (
	raid_id INT NOT NULL,
	raid_boss_id INT NOT NULL,
	FOREIGN KEY(raid_id) REFERENCES raids(id) ON DELETE CASCADE,
	FOREIGN KEY(raid_boss_id) REFERENCES raid_bosses(id) ON DELETE CASCADE,
	PRIMARY KEY(raid_id, raid_boss_id)
);
CREATE TABLE signup_boss_preference_mappings (
	signup_id INT NOT NULL,
	raid_boss_id INT NOT NULL,
	FOREIGN KEY(signup_id) REFERENCES signups(id) ON DELETE CASCADE,
	FOREIGN KEY(raid_boss_id) REFERENCES raid_bosses(id) ON DELETE CASCADE,
	PRIMARY KEY(signup_id, raid_boss_id)
);
