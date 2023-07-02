pub mod db;
pub mod opt;
pub mod web;

use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex, RwLock},
};

use color_eyre::{eyre::eyre, Result};
use rusqlite::Connection;
use trust_dns_proto::{
    op::message::Message,
    rr::{record_data::RData, Record},
    serialize::binary::BinDecodable,
};

#[derive(Debug, Default, Clone)]
pub struct QueryResult {
    name: String,
    records: Vec<Record>,
}

#[derive(Debug, Default, Clone)]
pub struct SimpleRecord {
    name: String,
    is_first: bool,
    data: String,
    tag: Option<String>,
}

impl QueryResult {
    pub fn to_simple_records(&self) -> Vec<SimpleRecord> {
        let mut result = vec![];
        let mut is_first = true;
        for r in &self.records {
            if let Some(rdata) = r.data() {
                match rdata {
                    RData::A(ip) => {
                        result.push(SimpleRecord {
                            name: self.name.to_string(),
                            is_first,
                            data: ip.to_string(),
                            tag: None,
                        });
                    }
                    RData::AAAA(ip) => {
                        result.push(SimpleRecord {
                            name: self.name.to_string(),
                            is_first,
                            data: ip.to_string(),
                            tag: None,
                        });
                    }
                    RData::CNAME(cname) => {
                        let s = cname.to_string();
                        let cname_str = s.trim_end_matches('.');
                        result.push(SimpleRecord {
                            name: self.name.to_string(),
                            is_first,
                            data: cname_str.to_string(),
                            tag: None,
                        });
                    }
                    RData::HTTPS(svcb) => {
                        use trust_dns_proto::rr::rdata::svcb::{
                            IpHint, SvcParamKey, SvcParamValue,
                        };
                        let has_ech = svcb
                            .svc_params()
                            .iter()
                            .any(|(k, _)| matches!(k, SvcParamKey::EchConfig));
                        let tag = if has_ech { "HTTPS ECH" } else { "HTTPS" };
                        for (_, v) in svcb.svc_params() {
                            match v {
                                SvcParamValue::Ipv4Hint(IpHint(ips)) => {
                                    for ip in ips {
                                        result.push(SimpleRecord {
                                            name: self.name.to_string(),
                                            is_first,
                                            data: ip.to_string(),
                                            tag: Some(tag.to_string()),
                                        });
                                        is_first = false;
                                    }
                                }
                                SvcParamValue::Ipv6Hint(IpHint(ips)) => {
                                    for ip in ips {
                                        result.push(SimpleRecord {
                                            name: self.name.to_string(),
                                            is_first,
                                            data: ip.to_string(),
                                            tag: Some(tag.to_string()),
                                        });
                                        is_first = false;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
            is_first = false;
        }

        result
    }

    pub fn show(&self) {
        let records = self.to_simple_records();
        for r in records {
            let arrow = if r.is_first { "=>" } else { "->" };
            if let Some(tag) = r.tag {
                println!("{} {} {} {}", r.name, arrow, tag, r.data);
            } else {
                println!("{} {} {}", r.name, arrow, r.data);
            }
        }
    }
}

fn process(packet: &[u8]) -> Option<QueryResult> {
    // 42 = 14 Ethernet header + 20 IPv4 header + 8 UDP header
    match Message::from_bytes(&packet[42..]) {
        Ok(msg) => {
            let qname;
            let name;
            match msg.queries().iter().next() {
                Some(q) => {
                    qname = q.name().to_string();
                    name = qname.trim_end_matches('.');
                }
                None => return None,
            };

            return Some(QueryResult {
                name: name.to_string(),
                records: msg.answers().to_owned(),
            });
        }
        Err(e) => {
            eprintln!("Error: {e:?}");
            None
        }
    }
}

pub fn run(opt: &opt::Opt, conn: Arc<Mutex<Connection>>) -> Result<()> {
    let device = if let Some(device_name) = &opt.device {
        pcap::Device::list()?
            .into_iter()
            .find(|d| d.name == *device_name)
            .ok_or_else(|| eyre!("device {} not found", device_name))?
    } else {
        pcap::Device::lookup()
            .expect("device lookup failed")
            .expect("no device available")
    };

    let mut cap = pcap::Capture::from_device(device)?
        .immediate_mode(true)
        .open()?;
    cap.filter(&opt.filter, true)?;

    let mut total_counts = 0;
    let mut statistics = BTreeMap::new();

    loop {
        match cap.next_packet() {
            Ok(packet) => {
                if let Some(qr) = process(&packet) {
                    qr.show();
                    total_counts += 1;
                    statistics
                        .entry(qr.name.to_string())
                        .and_modify(|e| *e += 1)
                        .or_insert(1);
                    let conn = conn.lock().unwrap();
                    crate::db::insert_query_result(&conn, &qr)?;
                }
            }
            Err(pcap::Error::TimeoutExpired) => {}
            Err(e) => return Err(e.into()),
        }
        if total_counts % 10 == 0 {
            dbg!(&statistics);
        }
    }
}
