use nom::{
    bytes::complete::take,
    combinator::{map, map_res},
    number::complete::{be_u16, be_u32},
    IResult, InputTake,
};

use crate::{
    error::{ParserError, ParserErrorType},
    indexed_input::IByteInput,
    traits::Parse,
};

use super::{class::RecordClass, data::RecordData, name::Name, rdata::*, types::RecordType};

/// ```text
/// The answer, authority, and additional sections all share the same
/// format: a variable number of resource records, where the number of
/// records is specified in the corresponding count field in the header.
/// Each resource record has the following format:
///                                     1  1  1  1  1  1
///       0  1  2  3  4  5  6  7  8  9  0  1  2  3  4  5
///     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
///     |                                               |
///     /                                               /
///     /                      NAME                     /
///     |                                               |
///     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
///     |                      TYPE                     |
///     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
///     |                     CLASS                     |
///     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
///     |                      TTL                      |
///     |                                               |
///     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
///     |                   RDLENGTH                    |
///     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--|
///     /                     RDATA                     /
///     /                                               /
///     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
/// ```
///
/// [RFC1035 4.1.3 Resource record format](https://www.rfc-editor.org/rfc/rfc1035.html#section-4.1.3)
pub struct Record {
    /// A domain name to which this resource record pertains
    name: Name,
    /// two octets containing one of the RR type codes. THis field specifies the meaning of the
    /// data in the RDATA field.
    rtype: RecordType,
    /// two octets which specify the class of the data in the RDATA field
    class: RecordClass,
    /// a 32 bit unsigned integer that specifies the time interval (in seconds) that the resource
    /// record may be cached before it should be discarded. Zero value are interpreted to mean that
    /// the RR can only be used for the transaction in progress, and should not be cached.
    ///
    /// Unsigned value [errata](https://www.rfc-editor.org/errata/eid2130)
    ///
    /// TTL value is the less significant 31 bits of the 32 bit TTL field. If the most significant
    /// bit is set, treat the entire value recieved as 0. [RFC2181 TTL](https://datatracker.ietf.org/doc/html/rfc2181#section-8)
    ///
    ttl: u32,
    /// a variable length string of octets that describes the resource. The format of this
    /// information varies according to the TYPE and CLASS of the resource record. For example if
    /// the TYPE is A and the CLASS is IN, the RDATA field is a 4 octet ARPA internet address.
    rdata: RecordData,
}

impl Record {
    fn parse_ttl(i: IByteInput) -> IResult<IByteInput, u32, ParserError> {
        const SIGN_MASK: u32 = 0x1 << 31;
        map(be_u32, |v| if v & SIGN_MASK == SIGN_MASK { 0 } else { v })(i)
    }

    fn parse_rdata<'a>(
        i: IByteInput<'a>,
        rtype: &RecordType,
        class: &RecordClass,
    ) -> IResult<IByteInput<'a>, RecordData, ParserError> {
        let (i, rd_length) = be_u16(i)?;
        let (i, rdata_buf) = take(rd_length)(i)?;

        let rdata = match (rtype, class) {
            (RecordType::A, RecordClass::IN) => RecordData::A(A::parse(rdata_buf)?.1),
            (RecordType::SOA, _) => RecordData::SOA(SOA::parse(rdata_buf)?.1),
            _ => todo!(),
        };

        return Ok((i, rdata));
    }
}

impl Parse for Record {
    fn parse(i: IByteInput) -> IResult<IByteInput, Self, ParserError> {
        let (i, name) = Name::parse(i)?;
        let (i, rtype) = RecordType::parse(i)?;
        let (i, class) = RecordClass::parse(i)?;
        let (i, ttl) = Self::parse_ttl(i)?;
        let (i, rdata) = Self::parse_rdata(i, &rtype, &class)?;

        Ok((
            i,
            Record {
                name,
                rtype,
                class,
                ttl,
                rdata,
            },
        ))
    }
}
