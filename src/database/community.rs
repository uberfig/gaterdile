use rocket_db_pools::Connection;
use sqlx::Error;

use crate::transmission::{
    JoinServerResult, Room, TransmissionCommunity, TransmissionServerMember,
};

use super::{db::DbConn, users::get_user_name};

pub async fn get_community_members(
    conn: &mut Connection<DbConn>,
    server_id: i64,
) -> Result<Vec<TransmissionServerMember>, Error> {
    let mut val = sqlx::query_as!(
        TransmissionServerMember,
        "SELECT * FROM community_members WHERE server_id = $1",
        server_id
    )
    .fetch_all(&mut ***conn)
    .await;

    match &mut val {
        Ok(y) => {
            for member in y {
                if member.nickname.is_none() {
                    let uname = get_user_name(conn, member.userid).await;
                    member.nickname = Some(uname.unwrap_or("unable to fetch".to_string()));
                }
            }
        }
        Err(x) => {
            println!("err in get server members");
            dbg!(&x);
        }
    }

    val
}

pub async fn get_community(
    conn: &mut Connection<DbConn>,
    id: i64,
) -> Result<TransmissionCommunity, Error> {
    sqlx::query_as!(
        TransmissionCommunity,
        "SELECT * FROM communities WHERE id = $1",
        id
    )
    .fetch_one(&mut ***conn)
    .await
}

/// gets all servers a user is a part of
pub async fn get_user_communities(
    conn: &mut Connection<DbConn>,
    uid: i64,
) -> Result<Vec<TransmissionCommunity>, Error> {
    let x = sqlx::query_as!(
        TransmissionServerMember,
        "SELECT * FROM community_members WHERE userid = $1",
        uid
    )
    .fetch_all(&mut ***conn)
    .await;

    let x = x.unwrap();

    let mut y = Vec::with_capacity(x.len());

    for i in x.into_iter() {
        let result = sqlx::query_as!(
            TransmissionCommunity,
            "SELECT * FROM communities WHERE id = $1",
            i.server_id
        )
        .fetch_one(&mut ***conn)
        .await;

        if result.is_err() {
            return Err(result.unwrap_err());
        }

        let result = result.unwrap();

        y.push(result)
    }

    return Ok(y);
}

/// gets all rooms in a community
pub async fn get_community_rooms(
    conn: &mut Connection<DbConn>,
    server_id: i64,
) -> Result<Vec<Room>, Error> {
    let a = sqlx::query_as!(Room, "SELECT * FROM rooms WHERE server = $1", server_id)
        .fetch_all(&mut ***conn)
        .await;

    a
}

pub async fn join_community(
    conn: &mut Connection<DbConn>,
    server_id: i64,
    userid: i64,
    nickname: Option<String>,
) -> JoinServerResult {
    let _result = sqlx::query!(
        "INSERT INTO community_members(server_id, userid, nickname) VALUES($1, $2, $3)",
        server_id,
        userid,
        nickname
    )
    .execute(&mut ***conn)
    .await;

    match _result {
        Ok(_x) => JoinServerResult::Success(server_id),
        Err(x) => {
            dbg!(x);
            JoinServerResult::Failure
        }
    }
}

/// creates a community and a general room and returns the community id
pub async fn create_community(
    conn: &mut Connection<DbConn>,
    creator: i64,
    name: String,
) -> Result<i64, Error> {
    let result = sqlx::query!(
        "INSERT INTO communities(nickname, owner, is_public) VALUES($1, $2, $3) RETURNING id",
        name,
        creator,
        false
    )
    .fetch_one(&mut ***conn)
    .await;

    let id = match result {
        Ok(x) => x.id,
        Err(x) => return Err(x),
    };

    let _x = join_community(conn, id, creator, None).await;

    dbg!(_x);

    //it is ok to ignore this as it is not a critical error if the general room failed to be created because of a disconnect to the db
    //if a user were to create a community and right before this line the connection breaks it would simply create a community with no rooms
    //and the user could just create a general room manually once the database has been fixed
    let _result = create_room(conn, creator, id, "general".to_string()).await;

    return Ok(id);
}

pub async fn is_admin(
    conn: &mut Connection<DbConn>,
    userid: i64,
    community: i64,
) -> Result<bool, Error> {
    let a = sqlx::query!(
        "SELECT owner as id FROM communities WHERE id = $1",
        community
    )
    .fetch_optional(&mut ***conn)
    .await;

    match a {
        Ok(x) => {
            if x.is_some_and(|x| x.id == Some(userid)) {
                return Ok(true);
            }
        }
        Err(x) => return Err(x),
    }

    let a = sqlx::query!(
        "SELECT * FROM roles JOIN role_members on roles.id = role_members.roleid WHERE community = $1 AND is_admin = true AND userid = $2 LIMIT 1",
        community, userid
    )
    .fetch_optional(&mut ***conn)
    .await;

    match a {
        Ok(x) => {
            if x.is_some() {
                return Ok(true);
            }
        }
        Err(_x) => {}
    }

    return Ok(false);
}

pub enum CreateRoomResult {
    Success(i64),
    Failure,
    NotAuthorised,
    InvalidName,
    NameTaken,
}
pub async fn create_room(
    conn: &mut Connection<DbConn>,
    creator: i64,
    community: i64,
    name: String,
) -> Result<CreateRoomResult, Error> {
    let admin = is_admin(conn, creator, community).await;

    match admin {
        Ok(x) => {
            if !x {
                return Ok(CreateRoomResult::NotAuthorised);
            }
        }
        Err(x) => return Err(x),
    }

    let result = sqlx::query!(
        "SELECT * FROM rooms WHERE server = $1 AND name = $2 LIMIT 1",
        community,
        name
    )
    .fetch_optional(&mut ***conn)
    .await;

    if result.is_err() {
        return Err(result.unwrap_err());
    }

    if result.is_ok_and(|x| x.is_some()) {
        return Ok(CreateRoomResult::NameTaken);
    }

    let result = sqlx::query!(
        "INSERT INTO rooms(server, name) VALUES($1, $2) RETURNING id",
        community,
        name
    )
    .fetch_one(&mut ***conn)
    .await;

    match result {
        Ok(x) => Ok(CreateRoomResult::Success(x.id)),
        Err(x) => Err(x),
    }
}

pub async fn get_room(conn: &mut Connection<DbConn>, room_id: i64) -> Result<Room, Error> {
    let a = sqlx::query_as!(Room, "SELECT * FROM rooms WHERE id = $1", room_id)
        .fetch_one(&mut ***conn)
        .await;

    a
}
