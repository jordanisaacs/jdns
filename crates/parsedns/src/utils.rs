use std::net::Ipv4Addr;

use nom::{combinator::map, number::complete::be_u32, IResult};

use crate::{error::ParserError, indexed_input::IByteInput, traits::Parse};

impl Parse for Ipv4Addr {
    fn parse(i: IByteInput) -> IResult<IByteInput, Self, ParserError> {
        map(be_u32, |addr| Ipv4Addr::from(addr))(i)
    }
}

#[derive(Clone, Copy)]
pub struct TTL(u32);

impl Parse for TTL {
    fn parse(i: IByteInput) -> IResult<IByteInput, Self, ParserError> {
        const SIGN_MASK: u32 = 0x1 << 31;

        map(be_u32, |v| {
            if v & SIGN_MASK == SIGN_MASK {
                TTL(0)
            } else {
                TTL(v)
            }
        })(i)
    }
}

impl From<TTL> for u32 {
    fn from(ttl: TTL) -> Self {
        ttl.0
    }
}
