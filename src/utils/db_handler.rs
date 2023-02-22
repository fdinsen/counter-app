use std::{io::{Write, SeekFrom, Seek, Read, ErrorKind}, fmt};

use rusqlite::{Connection, params, Result, Error, blob::Blob};
use bytes::{Bytes, BytesMut, BufMut};

const DB_PATH: &str = "count.db";

#[derive(Debug, Clone)]
pub struct Pokemon {
    pub id: i32,
    pub name: String,
    pub counter: i32,
}

impl Pokemon {
    pub fn update_counter(&mut self, new_counter: i32) {
        self.counter = new_counter;
    }
}

// type Result<T> = std::result::Result<T, DBError>;

// #[derive(Debug, Clone)]
// struct DBError {
//     msg: String,
// }

// impl fmt::Display for DBError {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "{:?}", self.msg)
//     }
// }


pub fn connect() -> Result<Connection> {
    let conn = Connection::open(DB_PATH)?;
    conn.execute(
        "create table if not exists counters (
             name text primary key,
             count integer not null
         )",
        [],
    )?;
    conn.execute(
        "create table if not exists sprites (
             name text primary key,
             img blob not null
        )", 
        [],
    )?;
    Ok(conn)
}

pub fn add_new_counter(name: &str) -> Result<()>{
    let conn = Connection::open(DB_PATH)?;
    match conn.execute(
        "INSERT INTO counters (name, count) VALUES (?1, ?2)",
            params![name, 0]
    ) {
        Ok(_) => Ok(()),
        Err(err) => Err(err),
    }
}

pub fn increment_counter(id: i32) -> Result<i32>{
    add_counter(id, 1)
}

pub fn add_counter(id: i32, amnt: i32) -> Result<i32>{
    if id == -1 {
        return Err(rusqlite::Error::InvalidQuery);
    }
    let conn = Connection::open(DB_PATH)?;
    let mut stmt = conn.prepare(
        "SELECT count FROM counters WHERE rowid = ?1")?;
    let counter 
        = stmt.query_row(params![id], |row| {
         row.get(0)
        });
    let count:i32 = match counter {
        Ok(r) => r,
        Err(e) => return Err(e),
    };
    conn.execute(
            "UPDATE counters SET count = ?1 WHERE rowid = ?2",
                params![count + amnt, id]
    )?;
    Ok(count+amnt)
}

pub fn read_counter(id: i32)-> Result<Pokemon> {
    if id == -1 {
        return Err(rusqlite::Error::InvalidQuery);
    }
    let conn = Connection::open(DB_PATH)?;
    let mut stmt = conn.prepare(
        "SELECT name, count FROM counters WHERE rowid = ?1")?;
    let counter 
        = stmt.query_row(params![id], |row| {
         Ok(Pokemon {
            id,
            name: row.get(0)?,
            counter: row.get(1)?,
         })
        });
    let counter = match counter{
        Ok(counter) => counter,
        Err(e) => return Err(e),
    };
    Ok(counter)
}

pub fn get_row_id(name: &str)-> Result<i32> {
    let conn = Connection::open(DB_PATH)?;
    let mut stmt = conn.prepare(
        "SELECT rowid FROM counters WHERE name = ?1")?;
    let counter 
        = stmt.query_row(params![name], |row| {
         row.get(0)
        });
    let counter = match counter{
        Ok(counter) => counter,
        Err(e) => return Err(e),
    };
    Ok(counter)
}

pub fn get_all_counters() -> Result<Vec<Pokemon>>{
    let conn = Connection::open(DB_PATH)?;
    let mut stmt = conn.prepare(
        "SELECT rowid, name, count FROM counters")?;
    let result = stmt.query_map([], |row| {
        Ok(Pokemon{
            id: row.get(0)?,
            name: row.get(1)?,
            counter: row.get(2)?,
        })
    })?;
    let mut counters:Vec<Pokemon> = Vec::new();
    for counter in result{
        if let Ok(c) = counter {counters.push(c); } // destructuring but ignoring all errors.
    };
    Ok(counters)
}

pub fn get_sprite_row_id(name: &str)-> Result<i32> {
    let conn = Connection::open(DB_PATH)?;
    let mut stmt = conn.prepare(
        "SELECT rowid FROM sprites WHERE name = ?1")?;
    let id 
        = stmt.query_row(params![name], |row| {
         row.get(0)
        });
    let id = match id{
        Ok(id) => id,
        Err(e) => return Err(e),
    };
    Ok(id)
}

pub fn save_sprite(name: &str, img: Bytes) -> Result<Bytes> {
    let conn = Connection::open(DB_PATH)?;
    let _ = conn.execute(
        "INSERT OR IGNORE INTO sprites (name, img) VALUES (?1, ZEROBLOB(22528)) RETURNING rowid",
            params![name]
    );
    let rowid = match get_sprite_row_id(&name) {
        Ok(rowid) => rowid,
        Err(err) => {println!("hi: {:?}", name);return Err(err);},
    };
    let mut blob = conn.blob_open(rusqlite::DatabaseName::Main,"sprites", "img", rowid.into(), false)?;
    match blob.write(&img) {
        Ok(_) => Ok(img),
        Err(_) => {println!("Failed.......");Ok(img)},
    }
}

pub fn get_sprite(name: &str) -> Result<Bytes> {
    let conn = Connection::open(DB_PATH)?;
    let mut stmt = conn.prepare("SELECT rowid FROM sprites WHERE name = ?1")?;
    let rowid:i32 = match stmt.query_row(params![name], |row| {
        row.get(0)
    }) {
        Ok(r) => r,
        Err(err) => return Err(err),
    };
    let mut blob = match conn.blob_open(rusqlite::DatabaseName::Main, "sprites", "img", rowid.into(), true) {
        Ok(r) => r,
        Err(err) => return Err(err),
    };
    blob.seek(SeekFrom::Start(0));
    let mut buf = [0u8; 22528];
    let bytes_read = blob.read(&mut buf[..]);
    match bytes_read {
        Ok(size) => {
            let mut bytes = BytesMut::new();
            bytes.reserve(size);
            bytes.put_slice(&buf);
            return Ok(bytes.freeze());
        },
        Err(err) => Err(rusqlite::Error::BlobSizeError),
    }
}