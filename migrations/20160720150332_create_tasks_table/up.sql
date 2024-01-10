CREATE TABLE users (
	"id"		INTEGER NOT NULL UNIQUE,
	"username"	TEXT NOT NULL UNIQUE,
	"nickname"	TEXT,
	"password"	TEXT,
	"salt"		TEXT,
	"sessions"	BLOB,	--vec of structs with session key, creation date, ip address
	PRIMARY KEY("id" AUTOINCREMENT)
);

CREATE TABLE servers (
	"id"		INTEGER NOT NULL UNIQUE,
	"nickname"	TEXT,
	"owner"		INTEGER NOT NULL,
	"channels"	BLOB,	--vec of structs with the name of the channels, their ids, and their descriptions
	"emojis"	BLOB,	--vec of optional structs with the name and attachment id of an emoji, delete attachment and set to none when deleting emoji
	PRIMARY KEY("id" AUTOINCREMENT)
);

CREATE TABLE attachments (
	"id"		INTEGER NOT NULL UNIQUE,
	"name"		TEXT,
	"owner"		INTEGER NOT NULL,
	"server"	INTEGER NOT NULL,
	"content"	BLOB,
	PRIMARY KEY("id" AUTOINCREMENT)
);

CREATE TABLE messages (
	"id"		INTEGER NOT NULL UNIQUE,
	"sender"	INTEGER NOT NULL,
	"server"	INTEGER NOT NULL,
	"channel"	INTEGER NOT NULL,
	"reply"		INTEGER, --null if not a reply, otherwise other message's id
	"text"		TEXT,
	"emoji"		BLOB,	--vec of structs with the name of the emoji and the emoji's server and id in the order they appear
	"sqltime" 	DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
	PRIMARY KEY("id" AUTOINCREMENT)
);

-- INSERT INTO tasks (description) VALUES ("demo task");
-- INSERT INTO tasks (description) VALUES ("demo task2");


