export class Transmission {
	data;
	transmission_type;
	constructor(transmission_type, value) {
		this.transmission_type = transmission_type
		this.data = value;
	}
}

export class JoinCommunity {
	JoinCommunity;
	constructor(id) {
		this.JoinCommunity = id;
	}
}

export class CreateCommunity {
	CreateCommunity;
	constructor(name) {
		this.CreateCommunity = name;
	}
}

export class CreateRoom {
	CreateRoom = [];
	constructor(id, name) {
		this.CreateRoom.push(id);
		this.CreateRoom.push(name);
	}
}

export class GetChannelTransmit {
	GetRoom;
	constructor(channel) {
		this.GetRoom = channel;
	}
}

export class Message {
	text;
	sender;
	server;
	channel;
	// reply = null;

	constructor(text, uid, server, channel, replying = null) {
		this.text = text;
		this.sender = uid;
		this.server = server;
		this.channel = channel;
		if (replying != null) {
			this.reply = replying;
		}
	}
}

export class TransmitMessage {
	SendMessage;
	constructor(text, uid, server, channel, replying = null) {
		this.SendMessage = new Message(text, uid, server, channel, replying)
	}
}

export class GetPriorMessages {
	GetPriorMessages;
	constructor(id) {
		this.GetPriorMessages = id;
	}
}

export class GetCommunity {
	GetCommunity;
	constructor(server_id) {
		this.GetCommunity = server_id;
	}
}

export class CreateUser {
	CreateUser;
	constructor(username, pass) {
		this.CreateUser = new UserAuth(username, pass);
	}
}

export class Auth {
	Auth;
	constructor(username, pass) {
		this.Auth = new UserAuth(username, pass);
	}
}

export class UserAuth {
	username;
	password;
	constructor(username, pass) {
		this.username = username;
		this.password = pass;
	}
}

export class Reaction {
	reaction;
	message_id;
	constructor(value, message_id) {
		this.reaction = value;
		this.message_id = message_id
	}
}