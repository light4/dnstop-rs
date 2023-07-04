use std::path::Path;

use chrono::{DateTime, Utc};
use color_eyre::Result;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::QueryResult;

const INIT_SQL: &str = include_str!("init.sql");

#[derive(Debug)]
struct Domain {
    name: String,
    records: Vec<String>,
}

#[derive(Debug)]
pub struct DomainRow {
    pub id: u64,
    pub name: String,
    pub records: String,
    pub create_dt: DateTime<Utc>,
}

pub fn init_db<P>(path: P) -> Result<Connection>
where
    P: AsRef<Path>,
{
    // let conn = Connection::open_in_memory()?;
    let conn = Connection::open(path)?;

    conn.execute(
        INIT_SQL,
        (), // empty list of parameters.
    )?;

    Ok(conn)
}

pub fn insert_query_result(conn: &Connection, qr: &QueryResult) -> Result<()> {
    let domain = Domain {
        name: qr.name.to_string(),
        records: qr
            .to_simple_records()
            .iter()
            .map(|r| &r.data)
            .cloned()
            .collect(),
    };
    conn.execute(
        "INSERT INTO tb_dns_domain (name, records) VALUES (?1, ?2)",
        (&domain.name, &domain.records.join(",")),
    )?;

    Ok(())
}

#[derive(Deserialize, Serialize)]
pub struct DomainCount {
    name: String,
    count: u64,
}

pub fn top_k(conn: &Connection, k: u64) -> Result<Vec<DomainCount>> {
    let mut result = vec![];
    let sql = format!(
        "SELECT name, COUNT(*) as row_count
    FROM tb_dns_domain
    GROUP BY name
    ORDER BY row_count DESC
    LIMIT {};",
        k
    );

    let mut stmt = conn.prepare(&sql)?;
    let record_iter = stmt.query_map([], |row| {
        Ok(DomainCount {
            name: row.get(0)?,
            count: row.get(1)?,
        })
    })?;
    for r in record_iter {
        result.push(r.unwrap());
    }

    Ok(result)
}

pub fn run() -> Result<()> {
    let conn = init_db("dnstop.db")?;

    let domain = Domain {
        name: "token.services.mozilla.com".to_string(),
        records: vec![
            "prod.tokenserver.prod.cloudops.mozgcp.net".to_string(),
            "34.107.141.31".to_string(),
        ],
    };
    conn.execute(
        "INSERT INTO tb_dns_domain (name, records) VALUES (?1, ?2)",
        (&domain.name, &domain.records.join(",")),
    )?;

    let mut stmt = conn.prepare("SELECT id, name, records, create_dt FROM tb_dns_domain")?;
    let domain_iter = stmt.query_map([], |row| {
        Ok(DomainRow {
            id: row.get(0)?,
            name: row.get(1)?,
            records: row.get(2)?,
            create_dt: row.get(3)?,
        })
    })?;

    for domain in domain_iter {
        println!("Found domain {:?}", domain.unwrap());
    }
    Ok(())
}
