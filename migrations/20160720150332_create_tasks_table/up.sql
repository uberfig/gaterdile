CREATE TABLE users (
	id 			INTEGER PRIMARY KEY AUTOINCREMENT UNIQUE,
	username	TEXT NOT NULL UNIQUE,
	nickname	TEXT,
	password	TEXT
);

-- CREATE TABLE usernames (
-- 	userid		INTEGER PRIMARY KEY NOT NULL UNIQUE,
-- 	username	TEXT NOT NULL UNIQUE
-- );

CREATE TABLE servers (
	id			INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL UNIQUE,
	nickname	TEXT,
	owner		INTEGER NOT NULL,
	FOREIGN KEY(owner) REFERENCES users(id)
	-- emojis		BLOB	--vec of optional structs with the name and attachment id of an emoji, delete attachment and set to none when deleting emoji
);

CREATE TABLE channels (
	id			INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL UNIQUE,
	server		INTEGER NOT NULL,
	name		TEXT,
	FOREIGN KEY(server) REFERENCES servers(id) ON DELETE CASCADE
);

CREATE TABLE attachments (
	id			INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL UNIQUE,
	name		TEXT,
	owner		INTEGER NOT NULL,
	server		INTEGER NOT NULL,
	content		BLOB,
	FOREIGN KEY(owner) REFERENCES users(id) ON DELETE CASCADE,
	FOREIGN KEY(server) REFERENCES servers(id) ON DELETE CASCADE
);

CREATE TABLE messages (
	id			INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL UNIQUE,
	sender		INTEGER NOT NULL,
	server		INTEGER NOT NULL,
	channel		INTEGER NOT NULL,
	reply		INTEGER, --null if not a reply, otherwise other message's id
	text		TEXT,
	timestamp	BIGINT,
	FOREIGN KEY(sender) REFERENCES users(id) ON DELETE CASCADE
	-- FOREIGN KEY(server) REFERENCES servers(id) ON DELETE CASCADE
	-- FOREIGN KEY(channel) REFERENCES channels(id) ON DELETE CASCADE
);

CREATE TABLE reactions (
	id			INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL UNIQUE,
	messageid	INTEGER NOT NULL,
	userid		INTEGER NOT NULL,
	emoji		INTEGER NOT NULL,
	FOREIGN KEY(messageid) REFERENCES messages(id) ON DELETE CASCADE,
	FOREIGN KEY(userid) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE server_members (
	id			INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL UNIQUE,
	server_id		INTEGER NOT NULL,
	userid			INTEGER NOT NULL,
	nickname	TEXT,
	-- FOREIGN KEY(server_id) REFERENCES servers(id) ON DELETE CASCADE,
	FOREIGN KEY(userid) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE channel_events (
	id				INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL UNIQUE,
	channel_id		INTEGER NOT NULL,
	timestamp		BIGINT NOT NULL,
	event_type		INTEGER NOT NULL,
	message			INTEGER,
	reaction		INTEGER,
	user			INTEGER,
	FOREIGN KEY(user) REFERENCES users(id) ON DELETE CASCADE
	FOREIGN KEY(reaction) REFERENCES reactions(id) ON DELETE CASCADE
	FOREIGN KEY(message) REFERENCES messages(id) ON DELETE CASCADE
	FOREIGN KEY(channel_id) REFERENCES channels(id) ON DELETE CASCADE
);