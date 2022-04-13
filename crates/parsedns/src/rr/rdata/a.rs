use std::net::Ipv4Addr;

use nom::{combinator::map, IResult};

use crate::{error::ParserError, indexed_input::IByteInput, traits::Parse};

/// ```text
///     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
///     |                    ADDRESS                    |
///     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
///
/// where:
///
/// Hosts that have multiple Internet addresses will have multiple A
/// records.
///
/// A records cause no additional section processing.  The RDATA section of
/// an A line in a master file is an Internet address expressed as four
/// decimal numbers separated by dots without any imbedded spaces (e.g.,
/// "10.2.0.52" or "192.0.5.6").
/// ```
/// [RFC1035 3.4.1: A RDATA format](https://datatracker.ietf.org/doc/html/rfc1035#section-3.4.1)
///
/// An internet specific RR
pub struct A(Ipv4Addr);

impl A {
    /// ```text
    /// ADDRESS         A 32 bit Internet address.
    /// ```
    pub fn address(&self) -> Ipv4Addr {
        self.0
    }
}

impl Parse for A {
    fn parse(i: IByteInput) -> IResult<IByteInput, Self, ParserError> {
        map(Ipv4Addr::parse, |addr| Self(addr))(i)
    }
}
