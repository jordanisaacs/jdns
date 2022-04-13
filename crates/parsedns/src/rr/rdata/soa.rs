use nom::{
    combinator::map,
    number::{complete::be_i32, streaming::be_u32},
    sequence::tuple,
    IResult,
};

use crate::{error::ParserError, indexed_input::IByteInput, rr::name::Name, traits::Parse};

///
///
pub struct SOA {
    mname: Name,
    rname: Name,
    serial: u32,
    refresh: i32,
    retry: i32,
    expire: i32,
    minimum: u32,
}

impl Parse for SOA {
    fn parse(i: IByteInput) -> IResult<IByteInput, Self, ParserError> {
        map(
            tuple((
                Name::parse,
                Name::parse,
                be_u32,
                be_i32,
                be_i32,
                be_i32,
                be_u32,
            )),
            |(mname, rname, serial, refresh, retry, expire, minimum)| Self {
                mname,
                rname,
                serial,
                refresh,
                retry,
                expire,
                minimum,
            },
        )(i)
    }
}
