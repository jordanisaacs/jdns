use nom::IResult;

use crate::{error::ParserError, indexed_input::IByteInput};

pub trait Encode {
    fn encode(&self) -> Self;
}

pub trait Parse: Sized {
    fn parse(i: IByteInput) -> IResult<IByteInput, Self, ParserError>;
}
