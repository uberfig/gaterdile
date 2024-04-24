import * as trans_types from "./transmission_types.js";

export async function get_room(serverConn, channel) {
	const val = new trans_types.Transmission("GetChannel", new trans_types.GetChannelTransmit(channel));
	serverConn.send(JSON.stringify(val));
	console.log(JSON.stringify(val))
}

export async function create_room(serverConn, server, name) {
	const val = new trans_types.Transmission("GetChannel", new trans_types.CreateRoom(server, name));
	serverConn.send(JSON.stringify(val));
}

export async function join_community(serverConn, server) {
	const val = new trans_types.Transmission("JoinServer", new trans_types.JoinCommunity(server));
	serverConn.send(JSON.stringify(val));
}

export async function create_community(serverConn, name) {
	const val = new trans_types.Transmission("CreateCommunity", new trans_types.CreateCommunity(name));
	serverConn.send(JSON.stringify(val));
}

export async function get_user_communities(serverConn) {
	const val = new trans_types.Transmission("GetUserCommunities", "GetUserCommunities");
	serverConn.send(JSON.stringify(val));
}

export async function get_old_messages(serverConn) {
	console.log("oldest_message"+oldest_message)
	var out = new trans_types.Transmission("GetPriorMessages", new trans_types.GetPriorMessages(oldest_message));
	serverConn.send(JSON.stringify(out));
}

export async function get_community(serverConn, server_id) {
	var out = new trans_types.Transmission("GetServer", new trans_types.GetCommunity(server_id));
	serverConn.send(JSON.stringify(out));
}

export async function send_message(text, server, channel, reply = null) {
	var message = new trans_types.TransmitMessage(text, server, channel, reply);
	var outgoing = new trans_types.Transmission("Message", message);
	serverConn.send(JSON.stringify(outgoing));
	console.log("sent, ", outgoing);
}

export async function auth(serverConn, username, password) {
	var creds = new trans_types.Auth(username, password);
	var val = JSON.stringify(new trans_types.Transmission("Auth", creds));
	serverConn.send(val);
}

export async function signup(serverConn, username, password) {
	var creds = new trans_types.CreateUser(username, password);
	var val = JSON.stringify(new trans_types.Transmission("CreateUser", creds));
	serverConn.send(val);
}