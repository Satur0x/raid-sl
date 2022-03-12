-- Your SQL goes here
CREATE TABLE signups (
	id SERIAL PRIMARY KEY,
	user_id INTEGER NOT NULL,
	raid_id INTEGER NOT NULL,
	FOREIGN KEY(raid_id) REFERENCES raids(id),
	FOREIGN KEY(user_id) REFERENCES users(id),
	UNIQUE (user_id, raid_id)
)
