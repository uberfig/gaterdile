let authenticated = false;
let uid = -1;
let subscribed_server = -1;
let subscribed_channel = -1;
let uname_map = {};
let oldest_message = null;
let loading = false;
let replying = false;
let reply_id = null;

async function send_clicked() {
	var input = document.getElementById("message_input").textContent.trim();
	// var input = document.getElementById("message_input").innerText;
	// var input = document.getElementById("message_input").innerHTML;
	// input = input.replace("<br>", "\n");


	document.getElementById("message_input").textContent = "";
	if (input != "") {
		send_message(input, 0, 0, reply_id);
	}
	end_replying();
}

async function get_connection() {
	return new WebSocket("ws://127.0.0.1:8000/ws", "test");
}

//display to the user that they disconnected from the server
//and attempt to reconnect at perodic intervals
function disconnected() {
	console.log("disconnected from server");
}

//display to the user that they are connected to the server
//also set that we are connected
function connected() {

}

function input_grab_focus() {
	document.getElementById("message_input").focus();
}

function set_replying(id) {
	if (id == reply_id) {
		end_replying();
		return;
	}
	input_grab_focus();
	end_replying();
	document.getElementById("input_replying").style.display = "flex";
	const message = document.getElementById(id)
	message.classList.add("message_selected");
	document.getElementById("reply_username").innerText = uname_map[message.dataset.sender]
	
	replying = true;
	reply_id = id;
}

function end_replying() {
	document.getElementById("input_replying").style.display = "none";
	if (reply_id != null) {
		document.getElementById(reply_id).classList.remove("message_selected");
	}
	replying = false;
	reply_id = null;
}

function reply_butt_func(id) {
	console.log(id);
	set_replying(id);
}

function clear_highlight(element) {
	element.classList.remove("message_highlight");
}

async function scroll_and_highlight(id) {
	const element = document.getElementById(id);
	element.scrollIntoView();
	element.classList.add("message_highlight");

	console.log("eeping");
	// await sleep(500);
	await new Promise(r => setTimeout(r, 500));
	console.log("wake");

	clear_highlight(element);
}

function create_message_element(message) {
	const parent = document.createElement("div");
	parent.classList.add("message");

	const hovermenu = document.createElement("div");
	hovermenu.classList.add("hovermenu");
	parent.appendChild(hovermenu);

	const menu_items = document.createElement("div");
	menu_items.classList.add("menu_items");
	hovermenu.appendChild(menu_items);

	//corner-up-left

	const reply_butt = document.createElement("button");
	const reply_icon = feather.icons["corner-up-left"].toSvg({ 'stroke-width': 2, 'color': '#ffffff' });
	reply_butt.insertAdjacentHTML("afterbegin", reply_icon);
	menu_items.appendChild(reply_butt);
	reply_butt.addEventListener('click', () => reply_butt_func(message.id));

	const reaction_butt = document.createElement("button");
	reaction_butt.classList.add("reaction_button");
	const emoji_icon = feather.icons.smile.toSvg({ 'stroke-width': 2, 'color': '#ffffff' });
	reaction_butt.insertAdjacentHTML("afterbegin", emoji_icon);
	menu_items.appendChild(reaction_butt);

	const more_butt = document.createElement("button");
	const more_icon = feather.icons["more-horizontal"].toSvg({ 'stroke-width': 2, 'color': '#ffffff' });
	more_butt.insertAdjacentHTML("afterbegin", more_icon);
	menu_items.appendChild(more_butt);

	if (message.reply != null) {
		const reply = document.createElement("button");
		reply.addEventListener('click', () => scroll_and_highlight(message.reply));
		const reply_preview_icon = feather.icons["corner-up-right"].toSvg({ 'stroke-width': 2, 'color': '#ffffff' });
		reply.insertAdjacentHTML("afterbegin", reply_preview_icon);
		parent.appendChild(reply);
	}

	const top = document.createElement("div");
	top.classList.add("message_top");
	parent.appendChild(top);

	const uname_ele = document.createElement("p");
	uname_ele.classList.add("username");
	let name = uname_map[message.sender];
	const uname = document.createTextNode(name);
	uname_ele.appendChild(uname);
	top.appendChild(uname_ele);

	const datetime = document.createElement("p");
	datetime.classList.add("datetime");
	var date = new Date(message.timestamp);
	var day = date.getDate();
	var hours = date.getHours();
	var minutes = "0" + date.getMinutes();
	var seconds = "0" + date.getSeconds();
	var formattedTime = document.createTextNode(day + "/" + (parseInt(date.getMonth()) + 1) + "/" + date.getFullYear() + " " + hours + ':' + minutes.slice(-2));
	datetime.appendChild(formattedTime);
	top.appendChild(datetime);

	const message_content = document.createElement("div");

	let lines = message.text.split("\n");
	for (let i = 0; i < lines.length; i++) {
		if (lines[i] == "" && i != 0 && i != lines.length - 1) {
			message_content.appendChild(document.createElement("br"));
			continue
		}
		const p_ele = document.createElement("p");
		const node = document.createTextNode(lines[i]);
		p_ele.appendChild(node)
		message_content.appendChild(p_ele);
	}

	parent.appendChild(message_content);

	parent.id = message.id
	parent.dataset.sender = message.sender;
	parent.dataset.timestamp = message.timestamp;

	twemoji.parse(parent);
	var img = message_content.querySelectorAll('img');
	for (let index = 0; index < img.length; index++) {
		const element = img[index];
		if (element.parentElement.innerText === "") {
			element.parentElement.classList = "bigimg";
		}
	}
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

function push_notification(username, channel, content) {
	if (document.hidden) {
		console.log("hidden");
		if (!("Notification" in window)) {
			// Check if the browser supports notifications
			console.log("This browser does not support desktop notification");
		} else if (Notification.permission === "granted") {
			// const notification = new Notification(username+" "+channel, { body: content, icon: img });
			const notification = new Notification(username + " " + channel, { body: content });
		}
	}
}

function handle_NewMessage(message) {
	const chat = document.getElementById("chat");

	if (oldest_message == null && message.NewMessages.length > 0) {
		oldest_message = message.NewMessages[0].id; //message.text
	}

	for (let i = 0; i < message.NewMessages.length; i++) {

		if (message.NewMessages[0].sender != uid) {
			let name = uname_map[message.NewMessages[0].sender];
			push_notification(name, "#general", message.NewMessages[0].text);
		}



		let para = create_message_element(message.NewMessages[i]);
		if (chat.lastChild != null && chat.lastChild.dataset != null) {

			if (chat.lastChild.dataset.sender == message.NewMessages[i].sender && message.NewMessages[i].timestamp - chat.lastChild.dataset.timestamp < 4000) {
				let above = para.querySelector(".username");
				above.parentElement.style.display = "none";
				chat.lastChild.style.paddingBottom = "0px"
			}

		}
		chat.appendChild(para);
		if (checkVisible(chat.lastChild)) {
			para.scrollIntoView();
		}

	}
}

function handle_event_NewMessage(message) {
	console.log("handling", message);

	const chat = document.getElementById("chat");

	if (oldest_message == null) {
		oldest_message = message.NewMessage.id; //message.text
	}

	if (message.sender != uid) {
		let name = uname_map[message.sender];
		push_notification(name, "#general", message.text);
	}

	let para = create_message_element(message.NewMessage);
	if (chat.lastChild != null && chat.lastChild.dataset != null) {

		if (chat.lastChild.dataset.sender == message.sender && message.timestamp - chat.lastChild.dataset.timestamp < 4000) {
			let above = para.querySelector(".username");
			above.parentElement.style.display = "none";
			chat.lastChild.style.paddingBottom = "0px"
		}

	}
	chat.appendChild(para);
	if (checkVisible(chat.lastChild)) {
		para.scrollIntoView();
	}
}

function handle_PriorMessages(message) {
	const chat = document.getElementById("chat");
	loading = false;

	for (let i = message.PriorMessages.length - 1; i >= 0; i--) {
		oldest_message = message.PriorMessages[i].id;
		let para = create_message_element(message.PriorMessages[i]);
		if (chat.firstChild != null && chat.firstChild.dataset != null) {

			if (chat.firstChild.dataset.sender == message.PriorMessages[i].sender) {
				let below = chat.firstChild.querySelector(".username");
				below.parentElement.style.display = "none";
				para.style.paddingBottom = "0px"
			}

		}
		chat.prepend(para);
	}
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

function handle_channelevent(event) {

	for (i in event.ChannelEvent) {
		console.log(event.ChannelEvent[i]);
		switch (event.ChannelEvent[i].event_type) {
			case "NewMessage":
				handle_event_NewMessage(event.ChannelEvent[i].data);
				break;
		}
	}

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
		case "ChannelEvent":
			handle_channelevent(event.data);
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
async function establish_connection() {
	serverConn = await get_connection();

	serverConn.onopen = (event) => {

	};

	serverConn.onmessage = (event) => {
		const val = JSON.parse(event.data);

		console.log("got:", val);

		handle_event(serverConn, val);
	};

	serverConn.onclose = (event) => {
		disconnected();
	}
}

function notifyMe() {
	if (!("Notification" in window)) {
		// Check if the browser supports notifications
		console.log("This browser does not support desktop notification");
	} else if (Notification.permission === "granted") {
		// Check whether notification permissions have already been granted;
		// if so, create a notification
		// const notification = new Notification("Hi there!");
		// …
	} else if (Notification.permission !== "denied") {
		// We need to ask the user for permission
		Notification.requestPermission();
		// .then((permission) => {
		// 	// If the user accepts, let's create a notification
		// 	if (permission === "granted") {
		// 		const notification = new Notification("Hi there!");
		// 		// …
		// 	}
		// });
	}
}

function login(e) {
	e.preventDefault();

	var form = document.getElementById("login_form")
	var formData = new FormData(form);
	formData = Object.fromEntries(formData);

	creds = new Auth(formData.username, formData.password);
	val = JSON.stringify(new Transmission("Auth", creds));
	serverConn.send(val);

	notifyMe();

	return false;
}

function signup(e) {
	e.preventDefault();

	var form = document.getElementById("signup_form")
	var formData = new FormData(form);
	formData = Object.fromEntries(formData);

	creds = new CreateUser(formData.username, formData.password);
	val = JSON.stringify(new Transmission("CreateUser", creds));
	serverConn.send(val);

	return false;
}

// establish_connection();

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
	if (checkVisible(document.getElementById("loader")) && loading == false) {
		loading = true;
		// alert("top");
		get_old_messages(serverConn);
	}
}

function initEvents() {
	console.log("init");
	document.getElementById("message_input").addEventListener("keydown", text_input_event, false);
	document.getElementById("message_input").addEventListener("keypress", prevent, false);
	document.getElementById("login_form").addEventListener("submit", login, false);
	document.getElementById("signup_form").addEventListener("submit", signup, false);

	feather.replace({ 'stroke-width': 2, 'color': '#ffffff' });
	establish_connection();
	prompt_auth();
}

window.onload = initEvents;