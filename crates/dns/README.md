# DNS Recursive Resolver

## A Recursive resolver

Sends requests with `recursion denied` flag to root name server. Then follows the responses until it receives an authoritative response.

##

## QNAME minimization

Sources:
* [What is QNAME minimization?](https://www.isc.org/blogs/qname-minimization-and-privacy/)
* [RFC7816 DNS Query Name Minimisation to Improve Privacy](https://datatracker.ietf.org/doc/html/rfc7816)

## DNS Encryption and DNSSEC

DNSSEC is essentially public key encryption to sign the DNS response. Works for resolver -> name server.

Encryption only works from the client -> resolver side. The resolver -> name server side is not encrypted. There is DNS over TLS which encrypts over UDP/TCP. There is also DNS over HTTPS which sends it over HTTP or HTTP/2 protocols.

There is a [draft](https://datatracker.ietf.org/doc/draft-ietf-dprive-unilateral-probing/) on encrypted recursive-to-authoritative DNS

Sources:
* [Statement on DNS Encryption](https://root-servers.org/media/news/Statement_on_DNS_Encryption.pdf)
* [Authoritative DNS-over-TLS Operational Considerations](https://tools.ietf.org/id/draft-hal-adot-operational-considerations-01.html)
* [DNS over TLS vs. DNS over HTTPS](https://www.cloudflare.com/learning/dns/dns-over-tls/)
* [The Camel's Back: Recursive to Authoritative DNS with Encryption](https://www.centr.org/news/blog/ietf110-camel-back.html)

## Cache Poisoning

DNS has a 16 bit transaction id, a response can be spoofed to poison the cache. To get additional bits to expand search space use random source ports.

Solutions:

- Randomize transaction id
    - Use a random number generator
- Source port randomization
    - Each query should send from a random preallocated port
- Mixed case queries (DNS 0x20)
    - Many DNS servers copy the domain name back as it was sent. Thus, mixed cases are preserved. Check if the mixed case is the same
    - Whitelist name servers that do not apply the standards correctly
- Randomizing choice of nameservers (but can band below RTT)

Match the source UDP port, transaction id, and question section. The tuple looks like: (udp port, transaction id, qname, qclass, qtype)


Sources:
* [CS 526 Topic 19: DNS Security](https://www.cs.purdue.edu/homes/ninghui/courses/526_Fall13/handouts/13_526_topic19.pdf)
* [Unbound DNS Cache Poisoning](https://nlnetlabs.nl/documentation/unbound/DNS-cache-poisoning-vulnerability/)
* [An Illustrated Guide to the Kaminsky DNS Vulnerability](http://unixwiz.net/techtips/iguide-kaminsky-dns-vuln.html)
* [use of Bit 0x20 in DNS Labels to Improve Transaction Identity](https://datatracker.ietf.org/doc/html/draft-vixie-dnsext-dns0x20-00)
* [DNS and the bit 0x20](https://hypothetical.me/short/dns-0x20/)
* [Increased DNS Forger Resistance Through 0x20-Bit Encoding](https://astrolavos.gatech.edu/articles/increased_dns_resistance.pdf)
* [https://developers.google.com/speed/public-dns/docs/security#mitigations](https://developers.google.com/speed/public-dns/docs/security)



## Authoritative Name Servers Selection

There are 13 authoritative name servers.

1. Round robin
2. Smoothed round trip time

Sources:
* [Authority Server Selection of DNS Caching Resolvers](https://irl.cs.ucla.edu/data/files/papers/res_ns_selection.pdf)
* [Recursive resolver authoritative nameserver selection](https://blog.apnic.net/2019/08/16/recursive-resolver-authoritative-nameserver-selection/)
* [Google Public DNS: Randomizing choice of name servers](https://developers.google.com/speed/public-dns/docs/security#randomizing_choice_of_name_servers)
