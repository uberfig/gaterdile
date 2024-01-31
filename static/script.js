let authenticated = false;
let uid = -1;
let subscribed_server = -1;
let subscribed_channel = -1;
let uname_map = {};
let oldest_message = null;
let loading = false;

class Reaction {
	reaction;
	message_id;
	constructor(value, message_id) {
		this.reaction = value;
		this.message_id = message_id
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

class Transmission {
	data;
	transmission_type;
	constructor(transmission_type, value) {
		this.transmission_type = transmission_type
		this.data = value;
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
		// this.reply = replying;
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

class GetServer {
	GetServer;
	constructor(server_id) {
		this.GetServer = server_id;
	}
}

async function get_old_messages(serverConn) {
	var out = new Transmission("GetPriorMessages", new GetPriorMessages(oldest_message));
	serverConn.send(JSON.stringify(out));
}

async function get_server(serverConn, server_id) {
	var out = new Transmission("GetServer", new GetServer(server_id));
	serverConn.send(JSON.stringify(out));
}

async function send_clicked() {
	var input = document.getElementById("message_input").value;
	document.getElementById("message_input").value = "";
	send_message(input, 0, 0);
}

async function send_message(text, server, channel) {
	var message = new TransmitMessage(text, server, channel);
	var outgoing = new Transmission("Message", message);
	serverConn.send(JSON.stringify(outgoing));
	console.log("sent, ", outgoing);
}

async function get_connection() {
	return new WebSocket("ws://127.0.0.1:8000/ws", "test");
}

//display to the user that they disconnected from the server
//and attempt to reconnect at perodic intervals
function disconnected() {

}

//display to the user that they are connected to the server
//also set that we are connected
function connected() {

}

class GetChannelTransmit {
	GetChannel = [];
	constructor(server, channel) {
		this.GetChannel.push(server);
		this.GetChannel.push(channel);
	}
}
async function get_channel(serverConn, server, channel) {
	const val = new Transmission("GetChannel", new GetChannelTransmit(server, channel));
	serverConn.send(JSON.stringify(val));
}

class JoinServer {
	JoinServer;
	constructor(id) {
		this.JoinServer = id;
	}
}
async function join_server(serverConn, server) {
	const val = new Transmission("JoinServer", new JoinServer(server));
	serverConn.send(JSON.stringify(val));
}

function create_message_element(message) {
	const parent = document.createElement("div");
	parent.classList.add("message");

	const uname_ele = document.createElement("p");
	let name = uname_map[message.sender];
	const uname = document.createTextNode(name + ":");
	uname_ele.appendChild(uname);
	uname_ele.style.color = "rgb(147, 240, 167)"
	uname_ele.classList = "uname";
	parent.appendChild(uname_ele);

	let lines = message.text.split("\n");
	for (let i = 0; i < lines.length; i++) {
		if (lines[i] == "") {
			parent.appendChild(document.createElement("br"));
			continue
		}
		const p_ele = document.createElement("p");
		const node = document.createTextNode(lines[i]);
		p_ele.appendChild(node)
		parent.appendChild(p_ele);
	}

	parent.id = message.id
	parent.dataset.sender = message.sender;

	return parent;
}

// function isHidden(el) {
//     var style = window.getComputedStyle(el);
//     return (style.display === 'none')
// }
function checkVisible(elm) {
	var rect = elm.getBoundingClientRect();
	var viewHeight = Math.max(document.documentElement.clientHeight, window.innerHeight);
	return !(rect.bottom < 0 || rect.top - viewHeight >= 0);
}

function handle_NewMessage(message) {
	const chat = document.getElementById("chat");

	if (oldest_message == null && message.NewMessages.length > 0) {
		oldest_message = message.NewMessages[0].id;
	}

	for (let i = 0; i < message.NewMessages.length; i++) {
		let para = create_message_element(message.NewMessages[i]);
		if (chat.lastChild != null && chat.lastChild.dataset != null) {

			if (chat.lastChild.dataset.sender == message.NewMessages[i].sender) {
				let above = para.querySelector(".uname");
				above.style.display = "none";
				chat.lastChild.style.paddingBottom = "0px"
			}

		}
		chat.appendChild(para);
		if (checkVisible(chat.lastChild)) {
			para.scrollIntoView();
		}

	}
}

function handle_PriorMessages(message) {
	const chat = document.getElementById("chat");
	loading = false;

	for (let i = message.PriorMessages.length -1; i >= 0; i--) {
		oldest_message = message.PriorMessages[i].id;
		let para = create_message_element(message.PriorMessages[i]);
		if (chat.firstChild != null && chat.firstChild.dataset != null) {

			if (chat.firstChild.dataset.sender == message.PriorMessages[i].sender) {
				let below = chat.firstChild.querySelector(".uname");
				below.style.display = "none";
				para.style.paddingBottom = "0px"
			}

		}
		chat.prepend(para);
	}

	loading = false;
}

function prompt_auth() {
	document.getElementById("auth_prompt").style.display = "flex";
}

function hide_auth() {
	document.getElementById("auth_prompt").style.display = "none";
}

async function handle_authrequest(serverConn, event) {
	prompt_auth();
}

function auth_failure(reason) {
	authenticated = false;
	uid = -1;

	console.log("failed auth because: ", reason);
	var display = document.getElementById("auth_failure");
	display.innerText = reason;
	display.style.display = "inline";
}

function auth_success(newid) {
	authenticated = true;
	uid = newid;

	var display = document.getElementById("auth_failure");
	display.innerText = "";
	display.style.display = "none";

	document.getElementById("auth_prompt").style.display = "none";
}

async function handle_AuthResult(serverConn, event) {

	if (event.data.AuthResult.hasOwnProperty("Success")) {
		console.log("Login Success");
		auth_success(event.data.AuthResult.Success);
		join_server(serverConn, 0);
		get_server(serverConn, 0);
		get_channel(serverConn, 0, 0);
	} else {
		auth_failure(event.data.AuthResult);
	}
}

async function handle_serverinfo(event) {
	event = event.ServerInfo;
	for (let index = 0; index < event.users.length; index++) {
		console.log(event.users[index].nickname);
		uname_map[event.users[index].userid] = event.users[index].nickname;
	}
	console.log(uname_map);
}

async function handle_event(serverConn, event) {
	// if event
	console.log("handleing event: ", event.transmission_type);

	switch (event.transmission_type) {
		case "RequestAuth":
			handle_authrequest(serverConn, event);
			break;
		case "AuthResult":
			handle_AuthResult(serverConn, event);
			break;
		case "NewMessages":
			handle_NewMessage(event.data);
			break;
		case "Reaction":

			break;
		case "ServerInfo":
			handle_serverinfo(event.data);
			break;
		case "PriorMessages":
			handle_PriorMessages(event.data);
			break;
		case "UserServers":

			break;
		case "JoinServerResult":

			break;
		case "CreateUserResult":

			break;
		case "UsernameAvailable":

			break;
	}

}

var serverConn;
async function test() {
	serverConn = await get_connection();

	serverConn.onopen = (event) => {
		// serverConn.send("Here's some text that the server is urgently awaiting!");
		// serverConn.send(JSON.stringify(new Transmission("Reaction",new Reaction("🙂", 0))))
		// send_message("hello", 0, 0);
	};


	// exampleSocket.send("Here's some text that the server is urgently awaiting!");
	// exampleSocket.
	serverConn.onmessage = (event) => {
		const val = JSON.parse(event.data);

		console.log("got:", val);

		handle_event(serverConn, val);
	};

	// exampleSocket.close();
}
function login() {
	var form = document.getElementById("login_form")
	var formData = new FormData(form);
	formData = Object.fromEntries(formData);

	creds = new Auth(formData.username, formData.password);
	val = JSON.stringify(new Transmission("Auth", creds));
	serverConn.send(val);

	return false;
}

function signup() {
	var form = document.getElementById("signup_form")
	var formData = new FormData(form);
	formData = Object.fromEntries(formData);

	creds = new CreateUser(formData.username, formData.password);
	val = JSON.stringify(new Transmission("CreateUser", creds));
	serverConn.send(val);

	return false;
}

test();

function text_input_event(evt) {
	if (evt.key == "Enter" && !evt.shiftKey) {
		send_clicked()
	}
}

function prevent(evt) {
	if (evt.key == "Enter" && !evt.shiftKey) {
		evt.preventDefault();
	}
}

async function check_scroll() {
	console.log("scrolling");
	if (checkVisible(document.getElementById("loader")) && loading == false){
		loading = true;
		// alert("top");
		get_old_messages(serverConn);
	}
}

// const input = document.querySelector("input");
function initEvents() {
	console.log("init");
	document.getElementById("message_input").addEventListener("keydown", text_input_event, false);
	document.getElementById("message_input").addEventListener("keypress", prevent, false);
	// document.getElementById("chat").scroll(function(){
	// 	if($(this).scrollTop() === 0){
	// 		 alert("top");   
	// 	}
	// });
	// document.getElementById("chat").addEventListener("onscroll", check_scroll, false);
}

// $("#contend").scroll(function(){
//     if($(this).scrollTop() === 0){
//          alert("top");   
//     }
// });

window.onload = initEvents;