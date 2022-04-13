use nom::{
    combinator::map,
    number::{complete::be_i32, streaming::be_u32},
    sequence::tuple,
    IResult,
};

use crate::{
    error::ParserError, indexed_input::IByteInput, rr::name::Name, traits::Parse, utils::TTL,
};

/// ```text
///     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
///     /                     MNAME                     /
///     /                                               /
///     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
///     /                     RNAME                     /
///     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
///     |                    SERIAL                     |
///     |                                               |
///     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
///     |                    REFRESH                    |
///     |                                               |
///     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
///     |                     RETRY                     |
///     |                                               |
///     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
///     |                    EXPIRE                     |
///     |                                               |
///     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
///     |                    MINIMUM                    |
///     |                                               |
///     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
///
/// where:
///
/// SOA records cause no additional section processing.
///
/// All times are in units of seconds.
///
/// Most of these fields are pertinent only for name server maintenance
/// operations.  However, MINIMUM is used in all query operations that
/// retrieve RRs from a zone.  Whenever a RR is sent in a response to a
/// query, the TTL field is set to the maximum of the TTL field from the RR
/// and the MINIMUM field in the appropriate SOA.  Thus MINIMUM is a lower
/// bound on the TTL field for all RRs in a zone.  Note that this use of
/// MINIMUM should occur when the RRs are copied into the response and not
/// when the zone is loaded from a master file or via a zone transfer.  The
/// reason for this provison is to allow future dynamic update facilities to
/// change the SOA RR with known semantics.
/// ```
/// [RFC1035 3.3.13: SOA RDATA format](https://datatracker.ietf.org/doc/html/rfc1035#section-3.3.13)
pub struct SOA {
    mname: Name,
    rname: Name,
    serial: u32,
    refresh: u32,
    retry: u32,
    expire: u32,
    minimum: TTL,
}

impl SOA {
    /// ```text
    /// MNAME           The <domain-name> of the name server that was the
    ///                 original or primary source of data for this zone.
    /// ```
    pub fn mname(&self) -> &Name {
        &self.mname
    }

    /// ```text
    /// RNAME           A <domain-name> which specifies the mailbox of the
    ///                 person responsible for this zone.
    /// ```
    pub fn rname(&self) -> &Name {
        &self.rname
    }

    /// ```text
    /// SERIAL          The unsigned 32 bit version number of the original copy
    ///                 of the zone.  Zone transfers preserve this value.  This
    ///                 value wraps and should be compared using sequence space
    ///                 arithmetic.
    /// ```
    pub fn serial(&self) -> u32 {
        self.serial
    }

    /// ```text
    /// REFRESH         A 32 bit time interval before the zone should be
    ///                 refreshed.
    /// ```
    pub fn refresh(&self) -> u32 {
        self.refresh
    }

    /// ```text
    /// RETRY           A 32 bit time interval that should elapse before a
    ///                 failed refresh should be retried.
    /// ```
    pub fn retry(&self) -> u32 {
        self.retry
    }

    /// ```text
    /// MINIMUM         The unsigned 32 bit minimum TTL field that should be
    ///                 exported with any RR from this zone.
    /// ```
    pub fn expire(&self) -> u32 {
        self.expire
    }

    /// ```text
    /// EXPIRE          A 32 bit time value that specifies the upper limit on
    ///                 the time interval that can elapse before the zone is no
    ///                 longer authoritative.
    /// ```
    pub fn minimum(&self) -> TTL {
        self.minimum
    }
}

impl Parse for SOA {
    fn parse(i: IByteInput) -> IResult<IByteInput, Self, ParserError> {
        map(
            tuple((
                Name::parse,
                Name::parse,
                be_u32,
                be_u32,
                be_u32,
                be_u32,
                TTL::parse,
            )),
            |(mname, rname, serial, refresh, retry, expire, minimum)| Self {
                mname,
                rname,
                serial,
                refresh,
                retry,
                expire,
                minimum: minimum.into(),
            },
        )(i)
    }
}
