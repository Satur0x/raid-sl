-- Your SQL goes here
CREATE TABLE raid_roles (
	raid_id INTEGER NOT NULL,
	role_id INTEGER NOT NULL,
	FOREIGN KEY(raid_id) REFERENCES raids(id),
	FOREIGN KEY(role_id) REFERENCES roles(id),
	PRIMARY KEY(raid_id, role_id)
)
