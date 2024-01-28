CREATE TABLE users (
	id 			INTEGER PRIMARY KEY AUTOINCREMENT UNIQUE,
	username	TEXT NOT NULL UNIQUE,
	nickname	TEXT,
	password	TEXT
);

CREATE TABLE usernames (
	userid		INTEGER PRIMARY KEY NOT NULL UNIQUE,
	username	TEXT NOT NULL UNIQUE
);

CREATE TABLE servers (
	id			INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL UNIQUE,
	nickname	TEXT,
	owner		INTEGER NOT NULL,
	emojis		BLOB	--vec of optional structs with the name and attachment id of an emoji, delete attachment and set to none when deleting emoji
);

CREATE TABLE channels (
	id			INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL UNIQUE,
	server		INTEGER NOT NULL,
	name		TEXT
);

CREATE TABLE attachments (
	id			INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL UNIQUE,
	name		TEXT,
	owner		INTEGER NOT NULL,
	server		INTEGER NOT NULL,
	content		BLOB
);

CREATE TABLE messages (
	id			INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL UNIQUE,
	sender		INTEGER NOT NULL,
	server		INTEGER NOT NULL,
	channel		INTEGER NOT NULL,
	reply		INTEGER, --null if not a reply, otherwise other message's id
	text		TEXT,
	timestamp	BIGINT
);

CREATE TABLE reactions (
	id			INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL UNIQUE,
	messageid	INTEGER NOT NULL,
	userid		INTEGER NOT NULL,
	emoji		INTEGER NOT NULL
);

CREATE TABLE server_members (
	id			INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL UNIQUE,
	server_id		INTEGER NOT NULL,
	userid			INTEGER NOT NULL,
	nickname	TEXT
);

