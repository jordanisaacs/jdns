use std::net::Ipv4Addr;

use dns_message_parser::{rr::NS, DomainName, DomainNameError};
pub use dns_message_parser::{
    rr::{A, RR},
    Dns, Flags, Opcode, RCode,
};

pub fn new_dns_packet() -> Dns {
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

pub fn get_random_a(packet: &Dns) -> Option<Ipv4Addr> {
    packet.answers.iter().find_map(|record| match record {
        RR::A(A { ipv4_addr, .. }) => Some(*ipv4_addr),
        _ => None,
    })
}

pub fn get_ns<'a>(
    packet: &'a Dns,
    qname: &'a str,
) -> impl Iterator<Item = (&'a DomainName, &'a DomainName)> {
    packet
        .authorities
        .iter()
        .filter_map(|record| match record {
            RR::NS(NS {
                domain_name,
                ns_d_name,
                ..
            }) => Some((domain_name, ns_d_name)),
            _ => None,
        })
        .filter(move |(domain, _)| qname.ends_with(&domain.to_string()))
}

pub fn get_resolved_ns<'a>(packet: &Dns, qname: &str) -> Option<Ipv4Addr> {
    get_ns(packet, qname).find_map(|(_, host)| {
        packet.additionals.iter().find_map(|record| match record {
            RR::A(A {
                domain_name,
                ipv4_addr,
                ..
            }) if domain_name == host => Some(*ipv4_addr),
            _ => None,
        })
    })
}

pub fn get_unresolved_ns<'a>(packet: &'a Dns, qname: &'a str) -> Option<&'a DomainName> {
    get_ns(packet, qname).next().map(|(_, host)| host)
}
