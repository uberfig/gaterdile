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
            owner -> BigSerial,
        }
    }
    table! {
        emojis {
            id      -> Nullable<BigSerial>,
            server  -> BigSerial,
            name    -> Text,
            attachmentid -> BigSerial,
        }
    }
    table! {
        channels (id, server) {
            id 		-> Nullable<BigSerial>,
            server 	-> BigSerial,
            name 	-> Text,
        }
    }
    table! {
        attachments {
            id -> BigSerial,
            name -> Text,
            owner -> BigSerial,
            server -> BigSerial,
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
            userid  -> Nullable<BigSerial>,
            roleid  -> Nullable<BigSerial>
        }
    }
    table! {
        reactions (id) {
            id -> BigSerial,
            userid 	-> BigSerial,
            messageid -> BigSerial,
            emoji	-> BigSerial,
        }
    }
    table! {
        server_members (server_id, userid) {
            // id  -> Nullable<Integer>,
            server_id -> BigSerial,
            userid  -> BigSerial,
            nickname -> Nullable<Text>,
        }
    }
    table! {
        channel_events {
            id -> Nullable<BigSerial>,
            channel_id -> BigSerial,
            server_id -> BigSerial,
            timestamp -> BigInt,
            event_type -> Integer,
            message -> Nullable<BigSerial>,
            reaction -> Nullable<BigSerial>,
            creator -> Nullable<BigSerial>,
            deleted -> Nullable<BigSerial>,
        }
    }
    table! {
        server_events {
            id -> Nullable<BigSerial>,
            server_id -> BigSerial,
            timestamp -> BigInt,
            event_type -> Integer,
            creator -> Nullable<BigSerial>,
            deleted -> Nullable<BigSerial>,
        }
    }
}
