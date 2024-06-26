import * as trans_methods from "./transmission_methods.js";

let authenticated = false;
let uid = -1;
let subscribed_server = -1;
let subscribed_channel = -1;
let uname_map = {};
let oldest_message = null;
let loading_prior = false;
let replying = false;
let reply_id = null;
let no_more_prior = false;

async function send_clicked() {
	var input = document.getElementById("message_input").textContent.trim();
	// var input = document.getElementById("message_input").innerText;
	// var input = document.getElementById("message_input").innerHTML;
	// input = input.replace("<br>", "\n");


	document.getElementById("message_input").textContent = "";
	if (input != "") {
		trans_methods.send_message(serverConn, input, uid, subscribed_server, subscribed_channel, reply_id);
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
	console.log("setting replying" + id);
	set_replying(id);
}

async function clear_highlight(element) {
	element.classList.remove("message_highlight");
	await new Promise(r => setTimeout(r, 400));
	element.classList.remove("fade_bg");
}

async function scroll_and_highlight(id) {
	if (id == null) {
		return;
	}

	if (document.getElementById(id) == null) {

		while (true) {
			await new Promise(r => setTimeout(r, 10));
			if (no_more_prior || document.getElementById(id) != null) {
				break;
			}
			if (loading_prior == true) {
				continue;
			}
			loading_prior = true;
			trans_methods.get_old_messages(serverConn);

		}
	}
	const element = document.getElementById(id);

	element.scrollIntoView();
	element.classList.add("fade_bg");
	element.classList.add("message_highlight");

	await new Promise(r => setTimeout(r, 600));

	await clear_highlight(element);
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
	reply_butt.classList.add("hovermenu_button");
	const reply_icon = feather.icons["corner-up-left"].toSvg({ 'stroke-width': 2, 'color': '#ffffff' });
	reply_butt.insertAdjacentHTML("afterbegin", reply_icon);
	menu_items.appendChild(reply_butt);
	reply_butt.addEventListener('click', () => reply_butt_func(message.id));

	const reaction_butt = document.createElement("button");
	reaction_butt.classList.add("reaction_button");
	reaction_butt.classList.add("hovermenu_button");
	const emoji_icon = feather.icons.smile.toSvg({ 'stroke-width': 2, 'color': '#ffffff' });
	reaction_butt.insertAdjacentHTML("afterbegin", emoji_icon);
	menu_items.appendChild(reaction_butt);

	const more_butt = document.createElement("button");
	more_butt.classList.add("hovermenu_button");
	const more_icon = feather.icons["more-horizontal"].toSvg({ 'stroke-width': 2, 'color': '#ffffff' });
	more_butt.insertAdjacentHTML("afterbegin", more_icon);
	menu_items.appendChild(more_butt);

	parent.dataset.reply = message.reply;
	parent.dataset.is_reply = message.is_reply;
	if (message.is_reply == true) {
		const reply = document.createElement("button");
		reply.classList.add("message_reply");
		reply.addEventListener('click', () => scroll_and_highlight(message.reply));
		const reply_preview_icon = feather.icons["corner-up-right"].toSvg({ 'stroke-width': 2, 'color': '#ffffff' });
		reply.insertAdjacentHTML("afterbegin", reply_preview_icon);

		const preview = document.createElement("p");

		if (reply != null) {
			const preview_uname = document.createElement("p");
			const preview_uname_text = document.createTextNode(uname_map[message.reply_uid]);
			preview_uname.appendChild(preview_uname_text);
			preview_uname.classList.add("username");
			reply.appendChild(preview_uname);

			const preview_text = document.createTextNode(message.reply_prev);
			preview.appendChild(preview_text);
		} else {
			const preview_text = document.createTextNode("Message Deleted");
			preview.appendChild(preview_text);
		}

		reply.appendChild(preview);

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
			console.log("This browser does not support notifications");
		} else if (Notification.permission === "granted") {
			// const notification = new Notification(username+" "+channel, { body: content, icon: img });
			const notification = new Notification(username + " " + channel, { body: content });
		}
	}
}

function is_timegroup(timestamp1, timestamp2) {
	let is_group = Math.abs(timestamp1 - timestamp2) < 4000;
	return is_group;
}

function can_merge_message(message, element) {
	return element.dataset.sender == message.sender && is_timegroup(message.timestamp, element.dataset.timestamp);
}

function handle_event_NewMessage(message) {

	const chat = document.getElementById("chat");

	if (oldest_message == null) {
		oldest_message = message.NewMessage.id; //message.text
	}

	if (message.sender != uid) {
		let name = uname_map[message.sender];
		push_notification(name, "#general", message.text);
	}

	let para = create_message_element(message.NewMessage);
	if (chat.lastChild != null && message.NewMessage.reply == null) {

		if (can_merge_message(message.NewMessage, chat.lastChild)) {
			let above = para.querySelector(".username");
			above.parentElement.style.display = "none";
			chat.lastChild.style.paddingBottom = "0px"
		}

	}
	if (chat.lastChild != null && checkVisible(chat.lastChild)) {
		chat.appendChild(para);
		para.scrollIntoView();
	} else {
		chat.appendChild(para);
	}
}

function handle_PriorMessages(message) {
	const chat = document.getElementById("chat");

	for (let i = message.PriorMessages.length - 1; i >= 0; i--) {
		oldest_message = message.PriorMessages[i].id;
		const curr_msg = message.PriorMessages[i].data.NewMessage;

		let para = create_message_element(curr_msg);
		if (chat.firstChild != null && chat.firstChild.dataset.reply == null) {

			if (can_merge_message(curr_msg, chat.firstChild)) {
				let below = chat.firstChild.querySelector(".username");
				below.parentElement.style.display = "none";
				para.style.paddingBottom = "0px"
			}

		}
		chat.prepend(para);
	}
	loading_prior = false;
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
		trans_methods.get_user_communities(serverConn);
		auth_success(event.data.AuthResult.Success);
	} else {
		auth_failure(event.data.AuthResult);
	}
}

function room_button_clicked(room_id){
	console.log(room_id);
	document.getElementById("chat").replaceChildren();
	trans_methods.get_room(serverConn, room_id);
	subscribed_channel = room_id;
}

function create_room_element(TransmissionChannel) {
	const parent = document.createElement("button");
	parent.classList.add("room_button");
	const text = document.createTextNode("# " + TransmissionChannel.name);
	parent.appendChild(text);
	parent.dataset.channelid = TransmissionChannel.id
	parent.addEventListener("click", () => room_button_clicked(TransmissionChannel.id));

	return parent;
}

async function handle_serverinfo(event) {
	event = event.ServerInfo;

	subscribed_server = event.server_data.id;
	document.getElementById("community_name").innerText = event.server_data.nickname;

	uname_map = {};
	for (let index = 0; index < event.users.length; index++) {
		console.log(event.users[index].nickname);
		uname_map[event.users[index].userid] = event.users[index].nickname;
	}
	console.log(uname_map);
	const room_container = document.getElementById("rooms");
	room_container.replaceChildren();

	for (let index = 0; index < event.channels.length; index++) {
		console.log(event.channels[index].name);
		room_container.appendChild(create_room_element(event.channels[index]));
	}
}

function handle_channelevent(event) {
	console.log(event.ChannelEvent);

	for (let index = 0; index < event.ChannelEvent.length; index++) {
		console.log(event.ChannelEvent[index]);
		// console.log(event.ChannelEvent[i]);
		switch (event.ChannelEvent[index].event_type) {
			case "NewMessage":
				console.log("NewMessage");
				handle_event_NewMessage(event.ChannelEvent[index].data);
				break;
		}
	}
}

function community_butt_pressed(community_id){
	console.log(community_id);
	trans_methods.get_community(serverConn, community_id);
}

function handle_user_communities(event) {
	var communities = document.getElementById("communities");
	var list = event.UserCommunities;

	for (let index = 0; index < list.length; index++) {
		const parent = document.createElement("button");
		parent.classList.add("community_butt");
		const text = document.createTextNode(list[index].nickname.charAt(0));
		parent.appendChild(text);
		parent.addEventListener("click", () => community_butt_pressed(list[index].id));
		parent.setAttribute("title", list[index].nickname);
		parent.id = "com_" + list[index].id;
		communities.appendChild(parent);
	}
}

async function handle_event(serverConn, event) {
	// if event
	console.log("handling event: ", event.transmission_type);

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
		case "ServerEvent":

			break;
		case "UserEvent":

			break;
		case "Reaction":

			break;
		case "ServerInfo":
			handle_serverinfo(event.data);
			break;
		case "PriorMessages":
			handle_PriorMessages(event.data);
			break;
		case "UserCommunities":
			handle_user_communities(event.data);
			break;
		case "JoinServerResult":

			break;
		case "CreateUserResult":

			break;
		case "UsernameAvailable":

			break;
		case "CreateCommunityResult":
			console.log(event);

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
	} else if (Notification.permission !== "denied") {
		// We need to ask the user for permission
		Notification.requestPermission();
	}
}

function login(e) {
	e.preventDefault();

	var form = document.getElementById("login_form");
	var formData = new FormData(form);
	formData = Object.fromEntries(formData);

	trans_methods.auth(serverConn, formData.username, formData.password);

	notifyMe();

	return false;
}

function signup(e) {
	e.preventDefault();

	var form = document.getElementById("signup_form")
	var formData = new FormData(form);
	formData = Object.fromEntries(formData);

	trans_methods.signup(serverConn, formData.username, formData.password);

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
	if (checkVisible(document.getElementById("loader")) && loading_prior == false && no_more_prior == false) {
		loading_prior = true;
		// alert("top");
		trans_methods.get_old_messages(serverConn);
	}
}

function create_community_prompt() {
	document.getElementById("create_comm_prompt").style.display = "flex";
}

function hide_community_prompt() {
	document.getElementById("create_comm_prompt").style.display = "none";
}

async function create_community(e) {
	e.preventDefault();

	var form = document.getElementById("create_comm_form")
	var formData = new FormData(form);
	formData = Object.fromEntries(formData);

	trans_methods.create_community(serverConn, formData.name);

	return false;
}

async function create_room(e) {
	e.preventDefault();

	return false;
}

function emoji_picker_clicked() {
	console.log("emoji picker clicked");
}

function initEvents() {
	console.log("init");
	document.getElementById("message_input").addEventListener("keydown", text_input_event, false);
	document.getElementById("message_input").addEventListener("keypress", prevent, false);

	document.getElementById("login_form").addEventListener("submit", login, false);
	document.getElementById("signup_form").addEventListener("submit", signup, false);

	document.getElementById("create_comm_form").addEventListener("submit", create_community, false);
	document.getElementById("cancel_create_comm_prompt").addEventListener("click", hide_community_prompt, false);
	document.getElementById("create_room_form").addEventListener("submit", create_room, false);

	document.getElementById("create_community_butt").addEventListener("click", create_community_prompt, false);
	document.getElementById("send_msg_button").addEventListener("click", send_clicked, false);
	document.getElementById("insert_emoji_button").addEventListener("click", create_community_prompt, false);

	feather.replace({ 'stroke-width': 2, 'color': '#ffffff' });
	establish_connection();
	prompt_auth();
}

window.onload = initEvents;