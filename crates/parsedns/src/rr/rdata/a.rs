use std::net::Ipv4Addr;

use nom::{combinator::map, number::complete::be_u8, sequence::tuple, IResult};

use crate::{error::ParserError, indexed_input::IByteInput, traits::Parse};

pub struct A(Ipv4Addr);

impl Parse for A {
    fn parse(i: IByteInput) -> IResult<IByteInput, Self, ParserError> {
        map(tuple((be_u8, be_u8, be_u8, be_u8)), |(a, b, c, d)| {
            Self(Ipv4Addr::new(a, b, c, d))
        })(i)
    }
}
