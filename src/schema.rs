pub mod schema {
    table! {
        users {
            id -> Nullable<Integer>,
            username -> Text,
            nickname -> Nullable<Text>,
            password -> Text,
        }
    }
    table! {
        servers {
            id -> Nullable<Integer>,
            nickname -> Text,
            owner -> Integer,
        }
    }
    table! {
        emojis {
            id      -> Nullable<Integer>,
            server  -> Integer,
            name    -> Text,
            attachmentid -> Integer,
        }
    }
    table! {
        channels (id, server) {
            id 		-> Nullable<Integer>,
            server 	-> Integer,
            name 	-> Text,
        }
    }
    table! {
        attachments {
            id -> Integer,
            name -> Text,
            owner -> Integer,
            server -> Integer,
            content -> Blob,
        }
    }
    table! {
        messages {
            id 		-> Nullable<Integer>,
            sender	-> Integer,
            server	-> Integer,
            channel -> Integer,
            // mention -> Blob,
            reply	-> Nullable<Integer>,
            text	-> Text,
            timestamp	-> BigInt,
        }
    }
    table! {
        mentions {
            id 		-> Integer, //message id
            userid  -> Nullable<Integer>,
            roleid  -> Nullable<Integer>
        }
    }
    table! {
        usernames (userid) {
            userid 	-> Integer,
            username-> Text,
        }
    }
    table! {
        reactions (id) {
            id -> Integer,
            userid 	-> Integer,
            messageid -> Integer,
            emoji	-> Integer,
        }
    }
    table! {
        server_members (server_id, userid) {
            // id  -> Nullable<Integer>,
            server_id -> Integer,
            userid  -> Integer,
            nickname -> Nullable<Text>,
        }
    }
}
