use rusqlite::{Connection, Result, params, Error};

const DB_PATH: &str = "count.db";

#[derive(Debug, Clone)]
pub struct Counter {
    pub id: i32,
    pub name: String,
    pub counter: i32,
}

// impl Counter{
//     pub fn new(&self, name: String) -> Self {
//         Counter{
//             name: name,
//             counter: 0,
//         }
//     }
//     pub fn to_string(&self) -> String {
//         String::from(format!("name: {:?}, counter: {:?}",self.name, self.counter))
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
    
    return Ok(conn);
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
                params![count + 1, id]
    )?;
    Ok(count+1)
}

pub fn read_counter(id: i32)-> Result<Counter> {
    if id == -1 {
        return Err(rusqlite::Error::InvalidQuery);
    }
    let conn = Connection::open(DB_PATH)?;
    let mut stmt = conn.prepare(
        "SELECT name, count FROM counters WHERE rowid = ?1")?;
    let counter 
        = stmt.query_row(params![id], |row| {
         Ok(Counter {
            id: id,
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

pub fn get_all_counters() -> Result<Vec<Counter>, Error>{
    let conn = Connection::open(DB_PATH)?;
    let mut stmt = conn.prepare(
        "SELECT rowid, name, count FROM counters")?;
    let result = stmt.query_map([], |row| {
        Ok(Counter{
            id: row.get(0)?,
            name: row.get(1)?,
            counter: row.get(2)?,
        })
    })?;
    let mut counters:Vec<Counter> = Vec::new();
    for counter in result{
        match counter {
            Ok(c) => {counters.push(c); },
            Err(_) => (),
        };
    };
    return Ok(counters);
}
