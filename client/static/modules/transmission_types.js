class Transmission {
	data;
	transmission_type;
	constructor(transmission_type, value) {
		this.transmission_type = transmission_type
		this.data = value;
	}
}

class JoinCommunity {
	JoinCommunity;
	constructor(id) {
		this.JoinCommunity = id;
	}
}

class GetChannelTransmit {
	GetRoom = [];
	constructor(server, channel) {
		this.GetRoom.push(server);
		this.GetRoom.push(channel);
	}
}

class Message {
	text;
	sender;
	server;
	channel;
	// reply = null;

	constructor(text, server, channel, replying = null) {
		this.text = text;
		this.sender = uid;
		this.server = server;
		this.channel = channel;
		if (replying != null) {
			this.reply = replying;
		}
	}
}

class TransmitMessage {
	SendMessage;
	constructor(text, server, channel, replying = null) {
		this.SendMessage = new Message(text, server, channel, replying)
	}
}

class GetPriorMessages {
	GetPriorMessages;
	constructor(id) {
		this.GetPriorMessages = id;
	}
}

class GetCommunity {
	GetCommunity;
	constructor(server_id) {
		this.GetCommunity = server_id;
	}
}

class CreateUser {
	CreateUser;
	constructor(username, pass) {
		this.CreateUser = new UserAuth(username, pass);
	}
}

class Auth {
	Auth;
	constructor(username, pass) {
		this.Auth = new UserAuth(username, pass);
	}
}

class UserAuth {
	username;
	password;
	constructor(username, pass) {
		this.username = username;
		this.password = pass;
	}
}

class Reaction {
	reaction;
	message_id;
	constructor(value, message_id) {
		this.reaction = value;
		this.message_id = message_id
	}
}