-- Your SQL goes here
CREATE TYPE raid_state AS ENUM ('created', 'open', 'closed', 'started', 'finished');
CREATE TABLE raids (
	id SERIAL PRIMARY KEY,
	title TEXT NOT NULL,
	date TIMESTAMP NOT NULL,
	state raid_state NOT NULL DEFAULT 'created'
)
