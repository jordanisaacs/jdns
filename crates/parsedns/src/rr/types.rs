use nom::{number::complete::be_u16, IResult};

use crate::{
    error::{ParserError, ParserErrorType},
    indexed_input::IByteInput,
    traits::Parse,
};

/// ```text
/// TYPE fields are used in resource records. Note that these types are a subset of QTYPES.
/// ```
///
/// [RFC1035 3.2.2: TYPE values](https://datatracker.ietf.org/doc/html/rfc1035#section-3.2.2)
pub enum RecordType {
    /// A host address [RFC1035 3.4.1: A RDATA Format](https://datatracker.ietf.org/doc/html/rfc1035#section-3.4.1)
    A,
    /// An authoritative name server [RFC1035 3.3.11 : NS RDATA format](https://datatracker.ietf.org/doc/html/rfc1035#section-3.3.11)
    NS,
    /// A mail destination (Obsolete - use MX) [RFC1035 3.3.4: MD RDATA format (Obsolete)](https://datatracker.ietf.org/doc/html/rfc1035#section-3.3.4)
    MD,
    /// A mail forwarder (Obsolete - use MX) [RFC1035 3.3.5: MF RDATA format (Obsolete)](https://datatracker.ietf.org/doc/html/rfc1035#section-3.3.5)
    MF,
    /// The canonical name for an alias [RFC1035 3.3.1: CNAME RDATA format](https://datatracker.ietf.org/doc/html/rfc1035#section-3.3.1)
    CNAME,
    /// A mailbox domain name (EXPERIMENTAL) [RFC1035](https://datatracker.ietf.org/doc/html/rfc1035#section-3.2.2)
    SOA,
    /// A mail group member (EXPERIMENTAL) [RFC1035](https://datatracker.ietf.org/doc/html/rfc1035#section-3.2.2)
    MB,
    /// A mail group member (EXPERIMENTAL)
    MG,
    /// A mail rename domain name (EXPERIMENTAL)
    MR,
    /// A null RR (EXPERIMENTAL)
    NULL,
    /// A well known service description
    WKS,
    /// A domain name pointer
    PTR,
    /// Host information
    HINFO,
    /// Mailbox or mail list information
    MINFO,
    /// Mail exchange
    MX,
    /// Text strings
    TXT,
    /// Responsible person [RFC1183 2.2: The Responsible Person RR](https://datatracker.ietf.org/doc/html/rfc1183#section-2.2)
    RP,
    /// AFS database location [RFC1183 1: AFS Data Base Location](https://datatracker.ietf.org/doc/html/rfc1183#section-1)
    AFSDB,
    /// X25 [RFC1183 3.1: The X25 RR](https://datatracker.ietf.org/doc/html/rfc1183#section-3.1)
    X25,
    /// ISDN (Integrated Service Digital Network) [RFC1183 3.2: The ISDN RR](https://datatracker.ietf.org/doc/html/rfc1183#section-3.2)
    ISDN,
    /// Route through [RFC1183 3.3: The Route Through RR](https://datatracker.ietf.org/doc/html/rfc1183#section-3.3)
    RT,
    NSAP,
    #[allow(non_camel_case_types)]
    NSAP_PTR,
    SIG,
    KEY,
    PX,
    GPOS,
    AAAA,
    LOC,
    NXT,
    EID,
    NIMLOC,
    SRV,
    ATMA,
    NAPTR,
    KX,
    CERT,
    A6,
    DNAME,
    SINK,
    OPT,
    APL,
    DS,
    SSHFP,
    NSEC,
    DNSKEY,
    DHCID,
    NSEC3,
    NSEC3PARAM,
    TLSA,
    SMIMEA,
    HIP,
    NINFO,
    RKEY,
    TALINK,
    CDS,
    CDNSKEY,
    OPENPGPKEY,
    CSYNC,
    ZONEMD,
    SVCB,
    Unknown(u16),
}

impl From<u16> for RecordType {
    fn from(value: u16) -> Self {
        match value {
            1 => Self::A,
            2 => Self::NS,
            3 => Self::MD,
            4 => Self::MF,
            5 => Self::CNAME,
            6 => Self::SOA,
            7 => Self::MB,
            8 => Self::MG,
            9 => Self::MR,
            10 => Self::NULL,
            11 => Self::WKS,
            12 => Self::PTR,
            13 => Self::HINFO,
            14 => Self::MINFO,
            15 => Self::MX,
            16 => Self::TXT,
            17 => Self::RP,
            18 => Self::AFSDB,
            19 => Self::X25,
            20 => Self::ISDN,
            21 => Self::RT,
            v => Self::Unknown(v),
        }
    }
}

impl Parse for RecordType {
    fn parse(i: IByteInput) -> IResult<IByteInput, Self, ParserError> {
        let (ir, v) = be_u16(i)?;

        match v.into() {
            Self::Unknown(v) => Err(nom::Err::Failure(ParserError {
                position: i.idx(),
                nom_kind: None,
                err_type: Some(ParserErrorType::UnrecognizedRecordType(v)),
            })),
            c => Ok((ir, c)),
        }
    }
}
