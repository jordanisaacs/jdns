use nom::error::{ErrorKind, ParseError};
use nom::{
    AsBytes, Err, IResult, InputIter, InputLength, InputTake, InputTakeAtPosition, Offset, Slice,
    ToUsize,
};

use core::ops::{RangeFrom, RangeTo};
use core::slice;
use std::ops::{AddAssign, Div, RangeFull, Shl, Shr};

pub type IByteInput<'a> = IndexedInput<&'a [u8], ()>;
pub type IBitInput<'a> = IndexedInput<&'a [u8], usize>;

#[derive(Debug, Clone, Copy)]
pub struct IndexedInput<T, B> {
    idx: usize,
    input: T,
    bit_offset: B,
}

impl<T> IndexedInput<T, ()> {
    pub fn new(input: T) -> Self {
        IndexedInput {
            idx: 0,
            input,
            bit_offset: (),
        }
    }

    pub fn idx(&self) -> usize {
        self.idx
    }

    pub fn input(&self) -> &T {
        &self.input
    }
}

impl<T> IndexedInput<T, ()>
where
    T: Slice<RangeFull>,
{
    pub fn to_bits(&self) -> IndexedInput<T, usize> {
        IndexedInput {
            idx: self.idx,
            input: self.input.slice(..),
            bit_offset: 0,
        }
    }
}

impl<T: AsBytes> IndexedInput<T, ()> {
    pub fn get_original_slice(&self) -> &[u8] {
        let self_bytes = self.input.as_bytes();
        let self_ptr = self_bytes.as_ptr();
        unsafe {
            assert!(self.idx <= isize::max_value() as usize, "Offset is too big");
            let orig_input_ptr = self_ptr.offset(-(self.idx as isize));
            slice::from_raw_parts(orig_input_ptr, self.idx + self_bytes.len())
        }
    }

    pub fn offset_original(&self, offset: usize) -> IndexedInput<&[u8], ()> {
        let orig = IndexedInput::new(self.get_original_slice());
        orig.slice(offset..)
    }
}

impl<T> IndexedInput<T, usize>
where
    T: Slice<RangeFrom<usize>> + InputIter<Item = u8> + InputLength + Offset,
{
    pub fn to_bytes(&self) -> IndexedInput<T, ()> {
        let next_input = if self.bit_offset % 8 != 0 {
            self.input.slice((1 + self.bit_offset / 8)..)
        } else {
            self.input.slice((self.bit_offset / 8)..)
        };
        let consumed_len = self.input.offset(&next_input);
        let next_offset = self.idx + consumed_len;

        IndexedInput {
            idx: next_offset,
            input: next_input,
            bit_offset: (),
        }
    }

    pub fn take<O, C, E: ParseError<Self>>(count: C) -> impl Fn(Self) -> IResult<Self, O, E>
    where
        C: ToUsize,
        O: From<u8> + AddAssign + Shl<usize, Output = O> + Shr<usize, Output = O>,
    {
        let count = count.to_usize();
        move |ii: IndexedInput<T, usize>| {
            if count == 0 {
                Ok((ii, 0u8.into()))
            } else {
                let cnt = (count + ii.bit_offset).div(8);
                if ii.input.input_len() * 8 < count + ii.bit_offset {
                    Err(Err::Error(E::from_error_kind(ii, ErrorKind::Eof)))
                } else {
                    let mut acc: O = 0_u8.into();
                    let mut offset: usize = ii.bit_offset;
                    let mut remaining: usize = count;
                    let mut end_offset: usize = 0;

                    for byte in ii.input.iter_elements().take(cnt + 1) {
                        if remaining == 0 {
                            break;
                        }
                        let val: O = if offset == 0 {
                            byte.into()
                        } else {
                            ((byte << offset) as u8 >> offset).into()
                        };

                        if remaining < 8 - offset {
                            acc += val >> (8 - offset - remaining);
                            end_offset = remaining + offset;
                            break;
                        } else {
                            acc += val << (remaining - (8 - offset));
                            remaining -= 8 - offset;
                            offset = 0;
                        }
                    }

                    let next_input = ii.input.slice(cnt..);
                    let consumed_len = ii.input.offset(&next_input);
                    let next_offset = ii.idx + consumed_len;
                    println!("{}", end_offset);

                    let ii = IndexedInput {
                        idx: next_offset,
                        input: next_input,
                        bit_offset: end_offset,
                    };

                    Ok((ii, acc))
                }
            }
        }
    }
}

impl<T> core::ops::Deref for IndexedInput<T, ()> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.input
    }
}

impl<'a> Into<IByteInput<'a>> for IBitInput<'a> {
    fn into(self) -> IByteInput<'a> {
        self.to_bytes()
    }
}

impl<'a> Into<IBitInput<'a>> for IByteInput<'a> {
    fn into(self) -> IBitInput<'a> {
        self.to_bits()
    }
}

impl<T: AsBytes> From<T> for IndexedInput<T, ()> {
    fn from(i: T) -> Self {
        Self::new(i)
    }
}

impl<T: AsBytes + PartialEq> PartialEq for IndexedInput<T, ()> {
    fn eq(&self, other: &Self) -> bool {
        self.input == other.input && self.idx == other.idx
    }
}

impl<T: AsBytes> AsBytes for IndexedInput<T, ()> {
    fn as_bytes(&self) -> &[u8] {
        self.input.as_bytes()
    }
}

impl<T: InputLength> InputLength for IndexedInput<T, ()> {
    fn input_len(&self) -> usize {
        self.input.input_len()
    }
}

impl<T: InputIter> InputIter for IndexedInput<T, ()> {
    type Item = T::Item;
    type Iter = T::Iter;
    type IterElem = T::IterElem;

    fn iter_indices(&self) -> Self::Iter {
        self.input.iter_indices()
    }

    fn iter_elements(&self) -> Self::IterElem {
        self.input.iter_elements()
    }

    fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        self.input.position(predicate)
    }

    fn slice_index(&self, count: usize) -> Result<usize, nom::Needed> {
        self.input.slice_index(count)
    }
}

impl<T, R> Slice<R> for IndexedInput<T, ()>
where
    T: Slice<R> + Offset + AsBytes + Slice<RangeTo<usize>>,
{
    fn slice(&self, range: R) -> Self {
        let next_input = self.input.slice(range);
        let consumed_len = self.input.offset(&next_input);
        let next_offset = self.idx + consumed_len;

        IndexedInput {
            idx: next_offset,
            input: next_input,
            bit_offset: (),
        }
    }
}

impl<T, B> InputTake for IndexedInput<T, B>
where
    Self: Slice<RangeFrom<usize>> + Slice<RangeTo<usize>>,
{
    fn take(&self, count: usize) -> Self {
        self.slice(..count)
    }

    fn take_split(&self, count: usize) -> (Self, Self) {
        (self.slice(count..), self.slice(..count))
    }
}

impl<T> InputTakeAtPosition for IndexedInput<T, ()>
where
    T: InputTakeAtPosition + InputLength + InputIter,
    Self: Slice<RangeFrom<usize>> + Slice<RangeTo<usize>> + Clone,
{
    type Item = <T as InputIter>::Item;

    fn split_at_position_complete<P, E: ParseError<Self>>(
        &self,
        predicate: P,
    ) -> IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        match self.split_at_position(predicate) {
            Err(Err::Incomplete(_)) => Ok(self.take_split(self.input_len())),
            res => res,
        }
    }

    fn split_at_position<P, E: ParseError<Self>>(&self, predicate: P) -> IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        match self.input.position(predicate) {
            Some(n) => Ok(self.take_split(n)),
            None => Err(Err::Incomplete(nom::Needed::new(1))),
        }
    }

    fn split_at_position1<P, E: ParseError<Self>>(
        &self,
        predicate: P,
        e: ErrorKind,
    ) -> IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        match self.input.position(predicate) {
            Some(0) => Err(Err::Error(E::from_error_kind(self.clone(), e))),
            Some(n) => Ok(self.take_split(n)),
            None => Err(Err::Incomplete(nom::Needed::new(1))),
        }
    }

    fn split_at_position1_complete<P, E: ParseError<Self>>(
        &self,
        predicate: P,
        e: ErrorKind,
    ) -> IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        match self.input.position(predicate) {
            Some(0) => Err(Err::Error(E::from_error_kind(self.clone(), e))),
            Some(n) => Ok(self.take_split(n)),
            None => {
                if self.input.input_len() == 0 {
                    Err(Err::Error(E::from_error_kind(self.clone(), e)))
                } else {
                    Ok(self.take_split(self.input_len()))
                }
            }
        }
    }
}
