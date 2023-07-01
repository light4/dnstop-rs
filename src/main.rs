use std::collections::BTreeMap;

use clap::Parser;
use color_eyre::{eyre::eyre, Result};
use trust_dns_proto::{
    op::message::Message,
    rr::{record_data::RData, Record},
    serialize::binary::BinDecodable,
};

#[derive(Parser)]
#[command(
    author,
    version,
    about,
    long_about = "Capture DNS requests and show their QNames"
)]
struct Opt {
    #[arg(long, help = "device")]
    device: Option<String>,
    #[arg(
        long,
        help = "pcap filter",
        default_value = "ip proto \\udp and src port 53"
    )]
    filter: String,
}

#[derive(Debug, Default, Clone)]
struct QueryResult {
    name: String,
    records: Vec<Record>,
}

fn show_rdata(name: &str, rdata: &RData, arrow: &str) {
    match rdata {
        RData::A(ip) => {
            println!("{name} {arrow} {ip}");
        }
        RData::AAAA(ip) => {
            println!("{name} {arrow} {ip}");
        }
        RData::CNAME(cname) => {
            let s = cname.to_string();
            let cname_str = s.trim_end_matches('.');
            println!("{name} {arrow} {cname_str}");
        }

        RData::HTTPS(svcb) => {
            use trust_dns_proto::rr::rdata::svcb::{IpHint, SvcParamKey, SvcParamValue};
            let has_ech = svcb
                .svc_params()
                .iter()
                .any(|(k, _)| matches!(k, SvcParamKey::EchConfig));
            let tag = if has_ech { "HTTPS ECH" } else { "HTTPS" };
            let mut is_first = true;
            for (_, v) in svcb.svc_params() {
                match v {
                    SvcParamValue::Ipv4Hint(IpHint(ips)) => {
                        for ip in ips {
                            let arrow = if is_first { "=>" } else { "->" };
                            println!("{name} {arrow} {tag} {ip}");
                            is_first = false;
                        }
                    }
                    SvcParamValue::Ipv6Hint(IpHint(ips)) => {
                        for ip in ips {
                            let arrow = if is_first { "=>" } else { "->" };
                            println!("{name} {arrow} {tag} {ip}");
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

fn show_query_result(qr: &QueryResult) {
    let mut is_first = true;
    for a in &qr.records {
        let arrow = if is_first { "=>" } else { "->" };
        if let Some(rdata) = a.data() {
            show_rdata(&qr.name, rdata, arrow);
        }
        is_first = false;
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

fn main() -> Result<()> {
    let opt = Opt::parse();
    let device = if let Some(device_name) = opt.device {
        pcap::Device::list()?
            .into_iter()
            .find(|d| d.name == device_name)
            .ok_or_else(|| eyre!("device {} not found", device_name))?
    } else {
        pcap::Device::lookup()
            .expect("device lookup failed")
            .expect("no device available")
    };
    dbg!(&device);

    let mut cap = pcap::Capture::from_device(device)?
        .immediate_mode(true)
        .open()?;
    cap.filter(&opt.filter, true)?;

    let mut statistics = BTreeMap::new();

    loop {
        match cap.next_packet() {
            Ok(packet) => {
                if let Some(qr) = process(&packet) {
                    show_query_result(&qr);
                    statistics
                        .entry(qr.name.to_string())
                        .and_modify(|e| *e += 1)
                        .or_insert(1);
                }
            }
            Err(pcap::Error::TimeoutExpired) => {}
            Err(e) => return Err(e.into()),
        }
        if statistics.len() % 10 == 0 {
            dbg!(&statistics);
        }
    }
}
