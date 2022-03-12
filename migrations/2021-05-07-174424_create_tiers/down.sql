-- This file should undo anything in `up.sql`
ALTER TABLE raids
	DROP COLUMN tier_id;

DROP TABLE tiers;
