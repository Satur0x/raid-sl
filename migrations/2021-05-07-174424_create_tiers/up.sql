-- Your SQL goes here
CREATE TABLE tiers (
	id SERIAL PRIMARY KEY,
	name TEXT UNIQUE NOT NULL
);

ALTER TABLE raids
	ADD COLUMN tier_id INT REFERENCES tiers(id);
