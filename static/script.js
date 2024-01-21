// const Transmission = {
//     Reaction
// }
class Reaction {
    reaction;
    message_id;
    constructor(value, message_id) {
        this.reaction = value;
        this.message_id = message_id
    }
}

class Auth {
    Auth;
    constructor(username, pass) {
        this.Auth = new Auth(username, pass);
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
    replying = null;
    emoji = null;
    timestamp;

    constructor(text, replying) {

    }
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

async function send_message(serverConn){
    let message = {

    }
}

async function get_context(serverConn){
    
}

async function handle_event(serverConn, event){
    // if event
    console.log("handleing event: ");
    console.log(event.transmission_type);
    console.log(event);

    if (event.transmission_type == "RequestAuth") {
        creds = new AuthTransmission("ivy", "123");
        val = JSON.stringify(new Transmission("Auth",creds));
        console.log("sending: ", val);
        serverConn.send(val);
    }

}

var serverConn;
async function test(){
    serverConn = await get_connection();

    serverConn.onopen = (event) => {
        // serverConn.send("Here's some text that the server is urgently awaiting!");
        // serverConn.send(JSON.stringify(new Transmission("Reaction",new Reaction("🙂", 0))))
    };


    // exampleSocket.send("Here's some text that the server is urgently awaiting!");
    // exampleSocket.
    serverConn.onmessage = (event) => {
        const val = JSON.parse(event.data);

        console.log("got:");
        console.log(event.data);
        console.log(val);

        handle_event(serverConn, val);
    };

// exampleSocket.close();
}
test();


