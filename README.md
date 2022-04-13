# Jordan's DNS

A (wip) simple dns server based on Xe's [dnsd](https://github.com/Xe/x/tree/master/cmd/dnsd) and the Rust DNS [guide](https://github.com/EmilHernvall/dnsguide)


## RFCs

[RFC1034: DNS Concepts & Algorithms](https://datatracker.ietf.org/doc/html/rfc1034)

[RFC1035: Base DNS Specification](https://datatracker.ietf.org/doc/html/rfc1035)

[RFC1183: Extra DNS RR (AFS, RP, X25, ISDN, RT)](https://datatracker.ietf.org/doc/html/rfc1183)

[RFC1706: DNS NSAP RR](https://datatracker.ietf.org/doc/html/rfc1706)

[RFC2136: DNS dynamic updates](https://datatracker.ietf.org/doc/html/rfc2136)

[RFC2181: DNS Spec Clarifications](https://www.rfc-editor.org/rfc/rfc2181)

[RFC2308: Negative caching of dns queries](https://datatracker.ietf.org/doc/html/rfc2308)

* Deprecates [RFC1034 4.3.1: Queries and responses](https://datatracker.ietf.org/doc/html/rfc2308)

[RFC3596: IPv6 DNS support](https://datatracker.ietf.org/doc/html/rfc3596)

[RFC4343: DNS case insensitivity clarifications](https://datatracker.ietf.org/doc/html/rfc4343)

[RFC6672: DNAME redirection](https://datatracker.ietf.org/doc/html/rfc6672)

* Obsoletes original DNAME spec in [RFC2672](https://datatracker.ietf.org/doc/html/rfc2672)

[RFC6761: Special-Use Domain Names](https://datatracker.ietf.org/doc/html/rfc6761)

[RFC6891: EDNS(0) extensions](https://datatracker.ietf.org/doc/html/rfc6891#section-5)

* Deprecates [RFC2673: binary labels](https://datatracker.ietf.org/doc/html/rfc6891#section-5) which should not be generated or passed (see [section 5](https://datatracker.ietf.org/doc/html/rfc6891#section-5))

[RFC7816: QName minimization](https://datatracker.ietf.org/doc/html/rfc7816)

[RFC7858: DNS over TLS](https://datatracker.ietf.org/doc/html/rfc7858)

[RFC8490: DNS Stateful operations (DSO)](https://datatracker.ietf.org/doc/html/rfc8490)

[RFC8767: Serve-stale for recursive resolvers](https://datatracker.ietf.org/doc/html/rfc8767)
