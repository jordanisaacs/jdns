pub mod dns;
pub mod udp;

use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};

use dns::packet::{new_dns_packet, Dns, RCode};

use bytes::Bytes;
use dashmap::{mapref::entry::Entry, DashMap};
use udp::{server::UdpServer, TxUdp};

struct DnsState {
    addr: SocketAddr,
    tx: TxUdp,
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
