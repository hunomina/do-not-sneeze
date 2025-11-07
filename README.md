# Do Not Sneeze

DNS implementation following [RFC 1035](https://datatracker.ietf.org/doc/html/rfc1035)

## Objective

Have a working standalone DNS server that can handle requests and deliver reliable responses.

## Modules

- ./common: domain structs
- ./decoding: decoder trait + implementation
- ./encoding: encoder trait + implementation
- ./storage: rr storage
- ./do-not-sneeze: server itself (with fallback server as parameter)
- ./client: wrap encoder + sender in cli tool

## Debug

When using `dig`: use `+noedns` to prevent OPT pseudo-record from EDNS(0) to be sent in the additional section (which is not supported by this implementation).
