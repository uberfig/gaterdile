let authenticated = false;
let uid = -1;
let subscribed_server = -1;
let subscribed_channel = -1;

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

async function send_clicked() {
	var input = document.getElementById("message_input").value;
	document.getElementById("message_input").value = "";
	send_message(input, 0, 0);
}

async function send_message(text, server, channel) {
	var message = new TransmitMessage(text, server, channel);
	var outgoing = new Transmission("Message", message);
	serverConn.send(JSON.stringify(outgoing));
	console.log("sent, ",JSON.stringify(outgoing));
}

async function get_connection(){
	return new WebSocket("ws://127.0.0.1:8000/ws", "test");
}

//display to the user that they disconnected from the server
//and attempt to reconnect at perodic intervals
function disconnected(){

}

//display to the user that they are connected to the server
//also set that we are connected
function connected() {

}

async function get_context(serverConn){
	
}

function handle_NewMessage(message){

}

function prompt_auth(){
	document.getElementById("auth_prompt").style.display = "flex";
}

function hide_auth(){
	document.getElementById("auth_prompt").style.display = "none";
}

async function handle_authrequest(serverConn, event){
	prompt_auth();
}

function auth_failure(reason){
	authenticated = false;
	uid = -1;

	console.log("failed auth because: ", reason);
	var display = document.getElementById("auth_failure");
	display.innerText = reason;
	display.style.display = "inline";
}

function auth_success(newid){
	authenticated = true;
	uid = newid;

	var display = document.getElementById("auth_failure");
	display.innerText = "";
	display.style.display = "none";

	document.getElementById("auth_prompt").style.display = "none";
}

async function handle_AuthResult(event){

	if (event.data.AuthResult.hasOwnProperty("Success")) {
		console.log("Login Success");
		auth_success(event.data.AuthResult.Success);
	} else {
		auth_failure(event.data.AuthResult);
	}
}

async function handle_event(serverConn, event){
	// if event
	console.log("handleing event: ");
	console.log(event.transmission_type);
	console.log(event);

	switch (event.transmission_type) {
		case "RequestAuth":
			handle_authrequest(serverConn, event);
			break;
		case "AuthResult":
			handle_AuthResult(event);
			break;
		case "NewMessage":
			handle_NewMessage(event.data);
			break;
		case "Reaction":

			break;
		case "CreateUserResult":

			break;
		case "UsernameAvailable":

			break;
	}

}

var serverConn;
async function test(){
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

		console.log("got:");
		console.log(val);

		handle_event(serverConn, val);
	};

// exampleSocket.close();
}
function login() {
	console.log("login");

	// data = document.getElementById("login_form")
	var form = document.getElementById("login_form")
	var formData = new FormData(form);
	formData = Object.fromEntries(formData);

	console.log(formData);
	console.log(formData.username);
	creds = new Auth(formData.username, formData.password);
	val = JSON.stringify(new Transmission("Auth",creds));
	serverConn.send(val);
	// console.log(Object.fromEntries(formData));	

	return false;
}

function signup() {
	console.log("signup")

	var form = document.getElementById("signup_form")
	var formData = new FormData(form);
	formData = Object.fromEntries(formData);

	console.log(formData);
	console.log(formData.username);
	creds = new CreateUser(formData.username, formData.password);
	val = JSON.stringify(new Transmission("CreateUser",creds));
	serverConn.send(val);

	return false;
}

test();