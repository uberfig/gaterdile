async function get_room(serverConn, server, channel) {
	const val = new Transmission("GetChannel", new GetChannelTransmit(server, channel));
	serverConn.send(JSON.stringify(val));
}

async function join_community(serverConn, server) {
	const val = new Transmission("JoinServer", new JoinCommunity(server));
	serverConn.send(JSON.stringify(val));
}

async function get_old_messages(serverConn) {
	console.log("oldest_message"+oldest_message)
	var out = new Transmission("GetPriorMessages", new GetPriorMessages(oldest_message));
	serverConn.send(JSON.stringify(out));
}

async function get_community(serverConn, server_id) {
	var out = new Transmission("GetServer", new GetCommunity(server_id));
	serverConn.send(JSON.stringify(out));
}

async function send_message(text, server, channel, reply = null) {
	var message = new TransmitMessage(text, server, channel, reply);
	var outgoing = new Transmission("Message", message);
	serverConn.send(JSON.stringify(outgoing));
	console.log("sent, ", outgoing);
}