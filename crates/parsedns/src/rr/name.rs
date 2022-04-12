use nom::{
    combinator::{map, map_res, peek},
    multi::length_data,
    number::complete::{be_u16, be_u8},
    IResult,
};

use tinyvec::TinyVec;
pub struct Label(pub String);

use crate::{
    error::{ParserError, ParserErrorType},
    indexed_input::IByteInput,
    traits::Parse,
};

const MAX_LABEL_LENGTH: u8 = 64;
const MAX_NAME_LENGTH: usize = 255;
const TYPE_MASK: u8 = 0xC0;
const ADDR_MASK: u16 = 0x3FF;

enum LabelType {
    Sequence,
    BackPointer,
    Root,
}

#[derive(Clone, Default)]
pub struct Name {
    is_fqdn: bool,
    label_data: TinyVec<[u8; 32]>,
    label_ends: TinyVec<[u8; 24]>,
}

impl Name {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn root() -> Self {
        let mut this = Self::new();
        this.is_fqdn = true;
        this
    }

    fn extend_name(&mut self, label: &[u8]) -> Result<(), ParserErrorType> {
        self.label_data.extend_from_slice(label);
        self.label_ends.push(self.label_data.len() as u8);
        if self.len() > MAX_NAME_LENGTH {
            Err(ParserErrorType::DomainNameTooLong(self.len()))
        } else {
            Ok(())
        }
    }

    pub fn len(&self) -> usize {
        let dots = if !self.label_ends.is_empty() {
            self.label_ends.len()
        } else {
            1
        };
        dots + self.label_data.len()
    }

    pub fn parse_label<'a>(
        &mut self,
        mut i: IByteInput<'a>,
        max_idx: Option<usize>,
    ) -> IResult<IByteInput<'a>, (), ParserError> {
        let name_start = i.idx();

        loop {
            if let Some(max_idx) = max_idx {
                // Protect against overlapping labels
                if i.idx() >= max_idx {
                    return Err(nom::Err::Failure(ParserError {
                        position: i.idx(),
                        nom_kind: None,
                        err_type: Some(ParserErrorType::LabelOverlapsWithOther {
                            label: i.idx(),
                            other: max_idx,
                        }),
                    }));
                }
            }

            let (_, parse_label) = Self::peek_type(i)?;
            match parse_label {
                LabelType::Sequence => {
                    let label;
                    (i, label) = Self::parse_seq_label(i)?;

                    self.extend_name(&label).map_err(|e| {
                        nom::Err::Failure(ParserError {
                            position: i.idx(),
                            nom_kind: None,
                            err_type: Some(e),
                        })
                    })?;
                }
                LabelType::BackPointer => {
                    let (_, offset) = Self::peek_ptr_offset(i)?;

                    self.parse_label(i.offset_original(offset), Some(name_start))?;
                    break;
                }
                LabelType::Root => {
                    let _x;
                    (i, _x) = be_u8(i)?;

                    break;
                }
            };
        }

        Ok((i, ()))
    }

    fn peek_ptr_offset<'a>(i: IByteInput<'a>) -> IResult<IByteInput<'a>, usize, ParserError> {
        // Ensure jump goes backward
        let mut parse_address = map_res(map(peek(be_u16), |b| (b & ADDR_MASK) as usize), |ptr| {
            if ptr < i.idx() {
                Ok(ptr)
            } else {
                Err(ParserErrorType::PointerNotPriorToLabel {
                    idx: i.idx(),
                    ptr: ptr as u16,
                })
            }
        });

        parse_address(i)
    }

    fn peek_type(i: IByteInput) -> IResult<IByteInput, LabelType, ParserError> {
        let parse_type = map_res(be_u8, |b| {
            if b == 0 {
                Ok(LabelType::Root)
            } else if (b & TYPE_MASK) == 0xC0 {
                Ok(LabelType::BackPointer)
            } else if (b & TYPE_MASK) == 0x00 {
                Ok(LabelType::Sequence)
            } else {
                Err(ParserErrorType::UnknownLabelType(b & TYPE_MASK))
            }
        });

        peek(parse_type)(i)
    }

    fn parse_seq_label(s: IByteInput) -> IResult<IByteInput, IByteInput, ParserError> {
        let parse_len = map_res(be_u8, |num| {
            if num >= MAX_LABEL_LENGTH {
                Err(ParserErrorType::LabelBytesTooLong(num.into()))
            } else {
                Ok(num)
            }
        });

        let mut parse_seq_label = length_data(parse_len);
        parse_seq_label(s)
    }
}

impl Parse for Name {
    fn parse(i: IByteInput) -> IResult<IByteInput, Self, ParserError> {
        let mut name = Name::root();
        let (i, _) = name.parse_label(i, None)?;
        Ok((i, name))
    }
}

#[cfg(test)]
mod tests {
    use crate::indexed_input::IByteInput;

    use super::Name;

    #[test]
    fn test_pointer_with_pointer_ending_labels() {}

    #[test]
    fn test_recursive_pointer() {
        // Points to an invalid beginning label marker
        let bytes = vec![0xC0, 0x01];
        let i = IByteInput::new(&bytes);

        let mut name = Name::root();
        assert!(name.parse_label(i, None).is_err());

        // Recurse back on itself
        let bytes = vec![0xC0, 0x00];
        let i = IByteInput::new(&bytes);
        assert!(name.parse_label(i, None).is_err());

        // Recurse back on itself
        let bytes = vec![0x01, 0x41, 0xC0, 0x00];
        let i = IByteInput::new(&bytes);
        assert!(name.parse_label(i, None).is_err());

        // Recurse by going past the end, then back to the beginning
        let bytes = vec![0xC0, 0x02, 0xC0, 0x00];
        let i = IByteInput::new(&bytes);
        assert!(name.parse_label(i, None).is_err());
    }
}
