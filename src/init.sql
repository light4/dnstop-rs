CREATE TABLE IF NOT EXISTS tb_dns_domain (
    id        INTEGER PRIMARY KEY,
    name      TEXT NOT NULL,
    records   TEXT NOT NULL,
    create_dt timestamp without time zone NOT NULL default (DATETIME('now'))
);
