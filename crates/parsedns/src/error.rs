use crate::indexed_input::IBitInput;

use super::indexed_input::IByteInput;
use nom::error::{ErrorKind, FromExternalError, ParseError};

#[derive(Debug)]
pub enum ParserErrorType {
    LabelBytesTooLong(u16),
    UnrecognizedLabelCode(u8),
    UnrecognizedClassCode(u16),
    UnrecognizedRecordType(u16),
    RDLengthTooLong(u16),
    DomainNameTooLong(usize),
    UnknownLabelType(u8),
    PointerNotPriorToLabel { idx: usize, ptr: u16 },
    LabelOverlapsWithOther { label: usize, other: usize },
}

#[derive(Debug)]
pub struct ParserError {
    pub position: usize,
    pub nom_kind: Option<ErrorKind>,
    pub err_type: Option<ParserErrorType>,
}

impl<'a> FromExternalError<IByteInput<'a>, ParserErrorType> for ParserError {
    fn from_external_error(input: IByteInput<'a>, kind: ErrorKind, e: ParserErrorType) -> Self {
        ParserError {
            position: input.idx(),
            nom_kind: Some(kind),
            err_type: Some(e),
        }
    }
}

impl<'a> ParseError<IBitInput<'a>> for ParserError {
    fn from_error_kind(input: IBitInput<'a>, kind: nom::error::ErrorKind) -> Self {
        ParserError {
            position: input.to_bytes().idx(),
            nom_kind: Some(kind),
            err_type: None,
        }
    }

    fn append(_: IBitInput<'a>, _: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}

impl<'a> ParseError<IByteInput<'a>> for ParserError {
    fn from_error_kind(input: IByteInput<'a>, kind: nom::error::ErrorKind) -> Self {
        ParserError {
            position: input.idx(),
            nom_kind: Some(kind),
            err_type: None,
        }
    }

    fn append(_: IByteInput<'a>, _: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}
