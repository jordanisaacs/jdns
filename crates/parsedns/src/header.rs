use nom::{combinator::map, IResult};

use crate::error::ParserError;
use crate::indexed_input::{IBitInput, IByteInput, IndexedInput};
use crate::traits::Parse;

/// RCodes appear at the top level of a DNS response (4 bits) but also inside TSIG RRs [RFC2845](https://datatracker.ietf.org/doc/html/rfc2845), TKEY RRs
/// [RFC2930](https://datatracker.ietf.org/doc/html/rfc2930),
/// and extended by OPT RRs [RFC6891]
///
/// Sizes: OPT RR (8 bit extension to 4 bit header), TSIG and TKEY RRs (16 bits)
///
/// From [RFC6895](https://datatracker.ietf.org/doc/html/rfc6895#section-2.3)
/// IANA [link](https://www.iana.org/assignments/dns-parameters/dns-parameters.xhtml#dns-parameters-6)
///
pub enum RCode {
    /// No error condition [RFC1035](https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.1)
    NoError,
    /// Format error - The name server was unable to interpret the query [RFC1035](https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.1)
    FormErr,
    /// Server failure - The name server was unable to process the query due to a problem with the
    /// name server [RFC1035](https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.1)
    ServFail,
    /// Name Error - Meaningful only for responses from an authoritative name server, this code
    /// signifies that the domain name referenced in the query does not exist [RFC1035](https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.1)
    NXDomain,
    /// Not Implemented - The name server does not support the requested kind of query [RFC1035](https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.1)
    NotImp,
    /// The name server refuses to perform the specified operation for policy reasons. For example,
    /// a name server may not wish to provide the information to a particular requester, or a name
    /// server may not wish to perform a particular operation (eg zone transfer) for particular
    /// data [RFC1035](https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.1)
    Refused,
    /// Some name that ought not to exist does exist. [RFC2136](https://datatracker.ietf.org/doc/html/rfc2136#section-2)
    YXDomain,
    /// Some RRset that ought not to exist does exist. [RFC2136](https://datatracker.ietf.org/doc/html/rfc2136#section-2)
    YXRRSet,
    /// Some RRset that ought to exist, does not exist. [RFC2136](https://datatracker.ietf.org/doc/html/rfc2136#section-2)
    NXRRSet,
    /// Means "Not Authoritative" [RFC2135](https://datatracker.ietf.org/doc/html/rfc2136#section-2) or "Not Authorized" [RFC2845](https://datatracker.ietf.org/doc/html/rfc2845).
    ///
    /// If appears as RCODE in the header of a DNS response without a TSIG RR, or TSIG RR having a zero error field,
    /// means "Not Authoritative"
    ///
    /// If appears as RCODE in the header of a DNS response that includes a TSIG RR with a non-zero
    /// error field means "Not Authorized"
    ///
    /// "Not Authoritative": The server is not authoritative for the zone named in the Zone Section. [RFC2136](https://datatracker.ietf.org/doc/html/rfc2136#section-2)
    ///
    /// "Not Authorized": [RFC2845](https://datatracker.ietf.org/doc/html/rfc2845)
    NotAuth,
    /// A name used in the prerequisite or update section is not within the zone denoted by the
    /// zone section. [RFC2136](https://datatracker.ietf.org/doc/html/rfc2136#section-2)
    NotZone,
    /// Server supports DSO but not the DSO-TYPE of the primary TLV in the DSO request message
    /// [RFC8490][https://www.rfc-editor.org/rfc/rfc8490.html#section-5.1.1]
    DSOTYPENI,
    /// Means BADVERS in OPT RR and BADSIG in TSIG RR [RFC6895](https://datatracker.ietf.org/doc/html/rfc6895#section-2.3)
    ///
    /// "Bad Vers": Responder does not implement version level of request [RFC6891](https://datatracker.ietf.org/doc/html/rfc6891#section-6.1.3)
    ///
    /// "Bad Sig": TSIG signature failure [RFC2845](https://datatracker.ietf.org/doc/html/rfc2845)
    BADSIGVERS,
    /// Key not recognized [RFC 2845](https://datatracker.ietf.org/doc/html/rfc2845)
    BADKEY,
    /// Signature out of time window [RFC2845](https://datatracker.ietf.org/doc/html/rfc2845)
    BADTIME,
    /// Duplicate key name [RFC2845](https://datatracker.ietf.org/doc/html/rfc2845)
    BADNAME,
    /// The server supports TKEY that but does not support the requested mode [RFC2930](https://datatracker.ietf.org/doc/html/rfc2930#section-2.5)
    BADMODE,
    /// Algorithm not supported
    BADALG,
    /// MAC is too short for local policy in force [RFC4635](https://datatracker.ietf.org/doc/html/rfc8945)
    BADTRUNC,
    /// Bad/missing server cookie [RFC7873](https://www.iana.org/go/rfc7873)
    BADCOOKIE,

    Unknown(u8),
}

impl From<u8> for RCode {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::NoError,
            1 => Self::FormErr,
            2 => Self::ServFail,
            3 => Self::NXDomain,
            4 => Self::NotImp,
            5 => Self::Refused,
            6 => Self::YXDomain,
            7 => Self::YXRRSet,
            8 => Self::NXRRSet,
            9 => Self::NotAuth,
            10 => Self::NotZone,
            11 => Self::DSOTYPENI,
            16 => Self::BADSIGVERS,
            17 => Self::BADKEY,
            18 => Self::BADTIME,
            19 => Self::BADNAME,
            20 => Self::BADMODE,
            21 => Self::BADALG,
            22 => Self::BADTRUNC,
            23 => Self::BADCOOKIE,
            other => Self::Unknown(other),
        }
    }
}

pub enum OpCode {
    /// A standard query [RFC1035](https://www.rfc-editor.org/rfc/rfc1035#section-4.1.1)
    Query,
    /// Inverse query (obsolete) [RFC3425](https://www.rfc-editor.org/rfc/rfc3425)
    IQuery,
    /// Server status request [RFC1035](https://www.rfc-editor.org/rfc/rfc1035#section-4.1.1)
    Status,
    /// Primary server advises secondary servers of data change [RFC1996](https://www.rfc-editor.org/rfc/rfc1996)
    Notify,
    /// Dynamic Update [RFC2136](https://www.rfc-editor.org/rfc/rfc2136)
    Update,
    /// DNS Stateful operations [RFC8490](https://www.rfc-editor.org/rfc/rfc8490.html)
    DSO,
    /// Not a known opcode
    Unknown(u8),
}

impl From<u8> for OpCode {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Query,
            1 => Self::IQuery,
            2 => Self::Status,
            4 => Self::Notify,
            5 => Self::Update,
            6 => Self::DSO,
            other => Self::Unknown(other),
        }
    }
}

///
/// Testing
///                                1  1  1  1  1  1
///  0  1  2  3  4  5  6  7  8  9  0  1  2  3  4  5
/// +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
/// |                      ID                       |
/// +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
/// |QR|   Opcode  |AA|TC|RD|RA| Z|AD|CD|   RCODE   |
/// +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
/// |                    QDCOUNT                    |
/// +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
/// |                    ANCOUNT                    |
/// +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
/// |                    NSCOUNT                    |
/// +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
/// |                    ARCOUNT                    |
/// +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
/// From [RFC 2535](https://datatracker.ietf.org/doc/html/rfc2535#section-6.1)
pub struct Header {
    /// A 16 bit identifier assigned by the program that generates any kind of query.
    /// This identifier is copied the corresponding reply and can be used by the
    /// requester to match up replies to outstanding queries. [RFC1035](https://www.rfc-editor.org/rfc/rfc1035)
    pub id: u16,
    /// A one bit field taht specifies whether this message is a query (0) or a response (1)
    /// [RFC1035](https://datatracker.ietf.org/doc/html/rfc2535#section-6.1)
    pub qr: bool,
    /// A four bit field that specifies the kind of query in this message. This value is set by the
    /// originator of a query and copied into the response. [RFC1035](https://datatracker.ietf.org/doc/html/rfc2535#section-6.1)
    pub opcode: OpCode,
    /// Authoritative Answer - this bit is valid in responses, and specifies that the responding
    /// name server is an authority for the domain name in question section.
    ///
    /// note: the contents of the answer section may have multiple owner names because of aliases.
    /// The AA bit corresponds to the name which matches the query name, or the first owner name in
    /// the answer section. [RFC1035](https://datatracker.ietf.org/doc/html/rfc2535#section-6.1)
    ///
    pub aa: bool,
    /// Truncation - specifies that this message was truncated due to length greater than that
    /// permitted on the transmission channel.[RFC1035](https://datatracker.ietf.org/doc/html/rfc2535#section-6.1)
    pub tc: bool,
    /// Recursion Desired - bit set in a query and copied into the response. If RD is set, directs
    /// name server to pursue the query recursively.[RFC1035](https://datatracker.ietf.org/doc/html/rfc2535#section-6.1)
    pub rd: bool,
    /// Recursion available - bit set or cleared in a response, and denotes whether recursive query
    /// support is available in the name server. [RFC1035](https://datatracker.ietf.org/doc/html/rfc2535#section-6.1)
    pub ra: bool,
    /// Authenticated data - bit indicates in a response that all data included in the answer and
    /// authority portion of ersponse has been authenticated by the server according to its
    /// policies. [RFC 2535](https://datatracker.ietf.org/doc/html/rfc2535#section-6.1)
    pub ad: bool,
    /// Checking disabled - bit indicates in a query that Pending (non-authenticated data) is
    /// acceptable to the resolver sending the query [RFC2535](https://datatracker.ietf.org/doc/html/rfc2535#section-6.1)
    pub cd: bool,
    pub z: bool,
    // Response code - 4 bit field set as part of responses. [RFC1035](https://datatracker.ietf.org/doc/html/rfc2535#section-6.1)
    pub rcode: RCode,
    // Question Count - unsigned 16 bit integer specifying the number of entries in the question
    // section. [RFC1035](https://datatracker.ietf.org/doc/html/rfc2535#section-6.1)
    pub qdcount: u16,
    // Answer Count - unsigned 16 bit integer specifying the number of entries in the answer
    // section. [RFC1035](https://datatracker.ietf.org/doc/html/rfc2535#section-6.1)
    pub ancount: u16,
    // Authority count - unsigned 16 bit integer specifying the number of name server resource
    // records in the authority records section. [RFC1035](https://datatracker.ietf.org/doc/html/rfc2535#section-6.1)
    pub nscount: u16,
    // Additional count - unsigned 16 bit integer specifying the number of name server resource
    // records in the additional records section. [RFC1035](https://datatracker.ietf.org/doc/html/rfc2535#section-6.1)
    pub arcount: u16,
}

impl Header {
    pub fn new() -> Self {
        Self {
            id: 0,

            qr: false,
            opcode: OpCode::Query,

            aa: false,
            tc: false,
            rd: false,
            ra: false,
            ad: false,
            cd: false,

            z: false,

            rcode: RCode::NoError,

            qdcount: 0,
            ancount: 0,
            nscount: 0,
            arcount: 0,
        }
    }
}

impl Parse for Header {
    fn parse(i: IByteInput) -> IResult<IByteInput, Self, ParserError> {
        let (i, id) = take_u16(i.into())?;
        let (i, qr) = take_bit(i)?;
        let (i, opcode) = take_nibble(i)?;
        let (i, aa) = take_bit(i)?;
        let (i, tc) = take_bit(i)?;
        let (i, rd) = take_bit(i)?;
        let (i, ra) = take_bit(i)?;
        let (i, z) = take_bit(i)?;
        let (i, ad) = take_bit(i)?;
        let (i, cd) = take_bit(i)?;
        let (i, rcode) = take_nibble(i)?;
        let (i, ancount) = take_u16(i)?;
        let (i, qdcount) = take_u16(i)?;
        let (i, nscount) = take_u16(i)?;
        let (i, arcount) = take_u16(i)?;

        Ok((
            i.into(),
            Self {
                id,
                qr,
                opcode: opcode.into(),
                aa,
                tc,
                rd,
                ra,
                z,
                ad,
                cd,
                rcode: rcode.into(),
                ancount,
                qdcount,
                nscount,
                arcount,
            },
        ))
    }
}

//impl Parse for Header {
//    //fn parse(i: IInput) -> IResult<IInput, Self, ParserError> {}
//}

pub fn take_bit(i: IBitInput) -> IResult<IBitInput, bool, ParserError> {
    map(IndexedInput::take(1u8), |bits: u8| bits != 0)(i)
}

pub fn take_nibble(i: IBitInput) -> IResult<IBitInput, u8, ParserError> {
    IndexedInput::take(4u8)(i)
}

pub fn take_u16(i: IBitInput) -> IResult<IBitInput, u16, ParserError> {
    IndexedInput::take(16u8)(i)
}

#[cfg(test)]
mod tests {
    use nom::bits::complete::take;
    use nom::IResult;

    use crate::header::IBitInput;
    use crate::indexed_input::IndexedInput;

    #[test]
    fn test_bit() {
        fn parser(input: IBitInput, count: usize) -> IResult<IBitInput, usize> {
            IndexedInput::take(count)(input)
        }
        let i = IndexedInput::new([1u8, 255, 3, 0, 5, 1, 9].as_ref()).to_bits();

        println!("{:?}", i);
        let i = parser(i, 8).unwrap();
        println!("{:?}", i);
        let i = parser(i.0, 3).unwrap();
        println!("{:?}", i);
        let i2 = i.0.to_bytes();
        println!("{:?}", i2);
        let i = parser(i.0, 4).unwrap();
        println!("{:?}", i);
        let i = parser(i.0, 4).unwrap();
        println!("{:?}", i);
    }
}
