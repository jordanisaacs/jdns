use std::net::Ipv4Addr;

use nom::IResult;

use crate::{error::ParserError, indexed_input::IByteInput, traits::Parse};

use super::rdata::*;

//pub struct Record {
//    pub name: String,
//    pub class: Class,
//    pub ttl: u32,
//    pub data: RecordData,
//}

pub enum RecordData {
    A(A),
}
