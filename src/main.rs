pub mod udp;
use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};

use bytes::Bytes;
use dashmap::{mapref::entry::Entry, DashMap};
use dns_message_parser::{Dns, Flags, Opcode, RCode};
use udp::{server::UdpServer, TxUdp};

struct DnsState {
    addr: SocketAddr,
    tx: TxUdp,
}

fn new_dns_packet() -> Dns {
    // https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.1
    // https://datatracker.ietf.org/doc/html/rfc2535 for AD & CD
    let flags = Flags {
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

    match entry {
        Entry::Occupied(entry) => {
            if addr != server {
                return;
            }

            let (_, dns_state) = entry.remove_entry();

            if !packet.flags.qr {
                println!("a query when supposed to be response");
                return;
            }

            println!("Response: {:?}", packet.encode().unwrap());

            dns_state
                .tx
                .send((packet.encode().unwrap().freeze(), dns_state.addr))
                .unwrap();
        }
        Entry::Vacant(entry) => {
            let mut lookup = new_dns_packet();
            lookup.flags.ra = true;

            if packet.questions.len() == 1 {
                entry.insert(DnsState {
                    addr,
                    tx: tx.clone(),
                });
            } else {
                lookup.flags.rcode = RCode::FormErr;
                tx.send((lookup.encode().unwrap().freeze(), addr)).unwrap();
                return;
            }

            lookup.id = packet.id;
            lookup.questions = packet.questions;
            lookup.flags.rd = packet.flags.rd;

            println!("Lookup {:?}", lookup);

            tx.send((lookup.encode().unwrap().freeze(), server))
                .unwrap();
        }
    };
}

#[tokio::main]
async fn main() {
    let init_state = Arc::new(DashMap::new());
    let server = UdpServer::new("0:2053", handle, init_state).unwrap();
    server.start().await.unwrap();
}
