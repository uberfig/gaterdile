async function get_channel(serverConn, server, channel) {
	const val = new Transmission("GetChannel", new GetChannelTransmit(server, channel));
	serverConn.send(JSON.stringify(val));
}

async function join_server(serverConn, server) {
	const val = new Transmission("JoinServer", new JoinServer(server));
	serverConn.send(JSON.stringify(val));
}

async function get_old_messages(serverConn) {
	var out = new Transmission("GetPriorMessages", new GetPriorMessages(oldest_message));
	serverConn.send(JSON.stringify(out));
}

async function get_server(serverConn, server_id) {
	var out = new Transmission("GetServer", new GetServer(server_id));
	serverConn.send(JSON.stringify(out));
}

async function send_message(text, server, channel) {
	var message = new TransmitMessage(text, server, channel);
	var outgoing = new Transmission("Message", message);
	serverConn.send(JSON.stringify(outgoing));
	console.log("sent, ", outgoing);
}