pub mod db_schema {
    table! {
        users {
            id -> Nullable<BigSerial>,
            username -> Text,
            nickname -> Nullable<Text>,
            password -> Text,
        }
    }
    table! {
        servers {
            id -> Nullable<BigSerial>,
            nickname -> Text,
            owner -> BigInt,
        }
    }
    table! {
        emojis {
            id      -> Nullable<BigSerial>,
            server  -> BigInt,
            name    -> Text,
            attachmentid -> BigInt,
        }
    }
    table! {
        channels (id, server) {
            id 		-> Nullable<BigSerial>,
            server 	-> BigInt,
            name 	-> Text,
        }
    }
    table! {
        attachments {
            id -> BigSerial,
            name -> Text,
            owner -> BigInt,
            server -> BigInt,
            content -> Blob,
        }
    }
    table! {
        messages {
            id 		-> Nullable<BigSerial>,
            sender	-> BigSerial,
            server	-> BigSerial,
            channel -> BigSerial,
            // mention -> Blob,
            reply	-> Nullable<BigSerial>,
            is_reply -> Bool,
            text	-> Text,
            timestamp	-> BigInt,
        }
    }
    table! {
        mentions {
            id 		-> BigSerial, //message id
            userid  -> Nullable<BigInt>,
            roleid  -> Nullable<BigInt>
        }
    }
    table! {
        reactions (id) {
            id -> BigSerial,
            userid 	-> BigInt,
            messageid -> BigInt,
            emoji	-> BigInt,
        }
    }
    table! {
        server_members (server_id, userid) {
            server_id -> BigInt,
            userid  -> BigInt,
            nickname -> Nullable<Text>,
        }
    }
    table! {
        channel_events {
            id -> Nullable<BigSerial>,
            channel_id -> BigInt,
            server_id -> BigInt,
            timestamp -> BigInt,
            event_type -> Integer,
            message -> Nullable<BigInt>,
            reaction -> Nullable<BigInt>,
            creator -> Nullable<BigInt>,
            deleted -> Nullable<BigInt>,
        }
    }
    table! {
        server_events {
            id -> Nullable<BigSerial>,
            server_id -> BigInt,
            timestamp -> BigInt,
            event_type -> Integer,
            creator -> Nullable<BigSerial>,
            deleted -> Nullable<BigSerial>,
        }
    }
    table! {
        user_events {
            id -> Nullable<BigSerial>,
            user_id -> BigInt,
            timestamp -> BigInt,
            event_type -> Integer,
            community -> Nullable<BigSerial>,
            deleted -> Nullable<BigSerial>,
        }
    }
}