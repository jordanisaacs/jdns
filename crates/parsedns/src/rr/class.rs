use nom::{
    combinator::{map, verify},
    error::Error,
    number::complete::be_u16,
    IResult,
};

use crate::{
    error::{ParserError, ParserErrorType},
    indexed_input::IByteInput,
    traits::Parse,
};

/// CLASS fields appear in resource records.
///
/// [RFC1035 3.2.4: CLASS values](https://datatracker.ietf.org/doc/html/rfc1035#section-3.2.4)
pub enum RecordClass {
    /// Internet
    IN,
    /// CSNET (Obsolete - used only for examples in some obsolete RFCs)
    CS,
    /// CHAOS class
    CH,
    /// Hesiod [Dyer 87]
    HS,
    // None, used in UPDATE queries to require that an RRset does not exist prior to the update.
    // [RFC2136](https://www.rfc-editor.org/rfc/rfc2136)
    NONE,
    /// Unknown record class
    Unknown(u16),
}

/// QCLASS fields appear in the question section of a query.  QCLASS values are a superset of CLASS values; every CLASS is a valid QCLASS.
///
/// [RFC1035 3.2.5: QCLASS values](https://www.rfc-editor.org/rfc/rfc1035.html#section-3.2.5)
pub enum RecordQClass {
    /// A general class
    RecordClass(RecordClass),
    /// "*" any class
    Any,
}

impl From<u16> for RecordQClass {
    fn from(value: u16) -> Self {
        match value {
            255 => Self::Any,
            v => Self::RecordClass(RecordClass::from(v)),
        }
    }
}

impl From<u16> for RecordClass {
    fn from(value: u16) -> Self {
        match value {
            1 => Self::IN,
            2 => Self::CS,
            3 => Self::CH,
            4 => Self::HS,
            254 => Self::NONE,
            v => Self::Unknown(v),
        }
    }
}

impl Parse for RecordClass {
    fn parse(i: IByteInput) -> IResult<IByteInput, Self, ParserError> {
        let (ir, v) = be_u16(i)?;

        match v.into() {
            Self::Unknown(v) => Err(nom::Err::Failure(ParserError {
                position: i.idx(),
                nom_kind: None,
                err_type: Some(ParserErrorType::UnrecognizedClassCode(v)),
            })),
            c => Ok((ir, c)),
        }
    }
}

impl Parse for RecordQClass {
    fn parse(i: IByteInput) -> IResult<IByteInput, Self, ParserError> {
        let (ir, v) = be_u16(i)?;

        match v.into() {
            Self::RecordClass(RecordClass::Unknown(v)) => Err(nom::Err::Failure(ParserError {
                position: i.idx(),
                nom_kind: None,
                err_type: Some(ParserErrorType::UnrecognizedClassCode(v)),
            })),
            c => Ok((ir, c)),
        }
    }
}
