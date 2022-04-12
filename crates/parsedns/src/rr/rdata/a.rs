use nom::IResult;

use crate::{error::ParserError, indexed_input::IByteInput, traits::Parse};

pub struct A {}

impl Parse for A {
    fn parse(i: IByteInput) -> IResult<IByteInput, Self, ParserError> {
        Ok((i, A {}))
    }
}
