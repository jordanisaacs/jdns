pub mod udp;
use std::{
    error::Error,
    net::{Ipv4Addr, SocketAddr},
    str::FromStr,
    sync::{Arc, Mutex},
};

use bytes::Bytes;
use dashmap::{mapref::entry::Entry, DashMap};
use dns_message_parser::{
    question::{QClass, QType, Question},
    Dns, DomainName, Flags, Opcode, RCode,
};
use socket2::Socket;
use tokio::net::UdpSocket;
use tokio::sync::oneshot;
use udp::{server::UdpServer, TxUdp};

struct DnsState {
    addr: SocketAddr,
    tx: TxUdp,
}

fn new_dns_packet() -> Dns {
    // https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.1
    // https://datatracker.ietf.org/doc/html/rfc2535 for AD & CD
    let mut flags = Flags {
        opcode: Opcode::Query, // Operation code
        qr: false,             // Query Response
        rd: false,             // Recursion Desired
        ra: false,             // Recursion Available
        tc: false,             // Truncation
        aa: false,             // Authoritative Answer
        ad: false,             // Authed Data
        cd: false,             // Checking Disabled
        rcode: RCode::NoError,
    };

    Dns {
        id: 0,
        flags,
        questions: Vec::new(),
        answers: Vec::new(),
        authorities: Vec::new(),
        additionals: Vec::new(),
    }
}

async fn handle(
    addr: SocketAddr,
    data: Bytes,
    state: Arc<DashMap<u16, DnsState>>,
    tx: TxUdp,
) -> () {
    let packet = Dns::decode(data).unwrap();
    let server: SocketAddr = (Ipv4Addr::new(8, 8, 8, 8), 53).into();

    let entry = state.entry(packet.id);
    let mut response = new_dns_packet();
    response.id = packet.id;
    response.flags.rd = true;
    response.flags.ra = true;
    response.flags.qr = true;

    match entry {
        Entry::Occupied(entry) => {
            if (addr != server) {
                return;
            }

            let (_, dns_state) = entry.remove_entry();

            if !packet.flags.qr {
                println!("a query when supposed to be response");
                return;
            }

            response.questions = packet.questions;
            response.answers = packet.answers;
            response.authorities = packet.authorities;
            response.additionals = packet.additionals;

            println!("Response: {:?}", response);

            dns_state
                .tx
                .send((response.encode().unwrap().freeze(), dns_state.addr))
                .unwrap();
        }
        Entry::Vacant(entry) => {
            if packet.questions.len() == 1 {
                entry.insert(DnsState {
                    addr,
                    tx: tx.clone(),
                });
            } else {
                response.flags.rcode = RCode::FormErr;
                tx.send((response.encode().unwrap().freeze(), addr))
                    .unwrap();
                return;
            }

            let mut lookup = new_dns_packet();

            lookup.id = packet.id;
            lookup.questions = packet.questions;
            lookup.flags.rd = packet.flags.rd;

            println!("Lookup {:?}", lookup);

            tx.send((lookup.encode().unwrap().freeze(), server))
                .unwrap();
        }
    };

    //if let Some(question) = request.questions.get(0) {
    //    if let Ok(result) = lookup(&question, tx).await {
    //    } else {
    //        flags.rcode = RCode::ServFail;
    //    }
    //} else {
    //    flags.rcode = RCode::FormErr;
    //}
}

#[tokio::main]
async fn main() {
    let init_state = Arc::new(DashMap::new());
    let server = UdpServer::new("0:2053", handle, init_state).unwrap();
    server.start().await.unwrap();

    //let qname = "google.com";

    //let server = ("8.8.8.8", 53);

    //let socket = UdpSocket::bind(("0.0.0.0", 43210)).unwrap();

    //let mut f = File::open("response_packet.txt").unwrap();

    //let mut buf = BytesMut::with_capacity(1024);
    //buf.resize(1024, 0);

    //let count = f.read(&mut buf[..]).unwrap();
    //buf.truncate(count);

    //let dns = Dns::decode(buf.freeze()).unwrap();
    //println!("{:?}", dns);
}
