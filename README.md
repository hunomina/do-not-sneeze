# Do Not Sneeze

A DNS server implementation written in Rust, following [RFC 1035](https://datatracker.ietf.org/doc/html/rfc1035). It is a fully functional DNS server that can handle DNS queries, serve records from an in-memory cache, and fall back to upstream DNS servers for unknown domains. The implementation follows RFC 1035 standards with a modular, trait-based architecture for extensibility.

## Objective of this project

Have a working standalone DNS server that can handle requests and deliver reliable responses.

For educational purposes, this project aims to use as little external dependencies as possible, implementing core DNS functionalities from scratch.

## Features

- **DNS Message Handling**: Encodes and decodes DNS messages following RFC 1035
- **Multi-threaded Server**: Handles concurrent DNS queries using thread-per-request model
- **UDP and TCP Transport**: Full support for both UDP (port 53) and TCP (port 53) protocols
- **Smart Caching**: Two-tier storage with in-memory cache and upstream DNS fallback
- **Upstream DNS Integration**: Automatically queries upstream DNS (e.g., 8.8.8.8) for unknown domains
- **Resource Record Support**: A, AAAA, TXT, CNAME, NS, and MX records fully implemented
- **EDNS(0) Support**: Extension Mechanisms for DNS (RFC 6891) with OPT pseudo-record handling
- **Response Truncation**: Automatic truncation of responses exceeding UDP size limits (512 bytes standard, 4096 bytes with EDNS)
- **RFC 1035 Compliant**: Proper handling of DNS headers, questions, and resource records
- **Domain Name Compression**: Efficient domain name encoding with label compression support

## Architecture

The project is organized into clear, modular components:

```
src/
├── common/           # Core DNS data structures
├── decoder/          # Binary to DNS message parsing
├── encoder/          # DNS message to binary serialization
├── storage/          # Record storage backends
├── server.rs         # UDP and TCP server on port 53
├── transport.rs      # Transport constants
├── utils.rs          # Bit manipulation utilities
└── main.rs           # Application entry point
```

### Design Patterns

- **Trait-Based Architecture**: `Decoder`, `Encoder`, and `ResourceRecordRepository` traits enable dependency injection and testing
- **Generic Storage**: Storage layer is generic over decoder/encoder implementations
- **Safe Concurrent Access**: Arc<Mutex<>> pattern for thread-safe storage operations
- **Error Propagation**: Result types throughout for proper error handling

## Supported Record Types

| Type | Code | Status |
|------|------|--------|
| **A** | 1 | ✅ Fully implemented (IPv4 addresses) |
| **AAAA** | 28 | ✅ Fully implemented (IPv6 addresses, RFC 3596) |
| **TXT** | 16 | ✅ Fully implemented (with RFC 1035 character-string format) |
| **CNAME** | 5 | ✅ Fully implemented (canonical name alias) |
| **NS** | 2 | ✅ Fully implemented (authoritative name server) |
| **MX** | 15 | ✅ Fully implemented (mail exchange with preference) |
| **OPT** | 41 | ✅ Fully implemented (EDNS(0) pseudo-record, RFC 6891) |
| PTR | 12 | ⚠️ Defined, encoding/decoding not implemented |
| SOA | 6 | ⚠️ Defined, encoding/decoding not implemented |

Plus 14 additional record types (HINFO, MINFO, WKS, SVCB, HTTPS, etc.)

## Getting Started

### Running the Server

```bash
# Run the server (may require sudo for port 53)
sudo cargo run

# Or run the compile the project and run the binary
cargo build --release
sudo ./target/release/do-not-sneeze
```

## Testing

### Run unit tests

```bash
cargo test
```

### Testing a running instance with dig

```bash
# Query A record (UDP)
dig @127.0.0.1 google.com A

# Query AAAA record (IPv6)
dig @127.0.0.1 google.com AAAA

# Query TXT record
dig @127.0.0.1 google.com TXT

# Query CNAME record
dig @127.0.0.1 www.example.com CNAME

# Query NS record
dig @127.0.0.1 example.com NS

# Query MX record
dig @127.0.0.1 example.com MX

# Force TCP transport
dig @127.0.0.1 google.com A +tcp

# Force EDNS(0) support (server responds with OPT record)
dig @127.0.0.1 google.com A +edns=0

# Disable EDNS if needed
dig @127.0.0.1 google.com A +noedns
```

**Note**: The server fully supports EDNS(0). When a client sends an OPT record, the server will respond with an OPT record indicating support for larger UDP payloads (4096 bytes). TCP transport is also fully supported with proper message framing (2-byte length prefix per RFC 1035).

## How It Works

### Request Flow

1. **Listen**: UDP or TCP socket receives DNS query on port 53
   - **UDP**: Direct datagram reception
   - **TCP**: Read 2-byte length prefix, then read message body
2. **Decode**: Binary message parsed into structured DNS Message
3. **Query Storage**:
   - Check in-memory cache first
   - If not found, query upstream DNS (8.8.8.8)
   - Cache upstream responses for future queries
4. **Format Response**: Original message converted to response with answers
5. **Encode**: DNS response serialized back to binary format
6. **Check Size** (UDP only): Verify response fits within UDP size limits
   - Standard DNS: 512 bytes (RFC 1035)
   - With EDNS(0): Up to 4096 bytes (configurable via OPT record)
   - If too large: Truncate response (set TC flag, clear all answer/authority/additional sections)
7. **Send**: Response sent back to client
   - **UDP**: Direct datagram transmission
   - **TCP**: Send 2-byte length prefix followed by message body

### Response Truncation

When a DNS response exceeds the maximum allowed UDP message size, the server automatically truncates it according to RFC 1035:

- **Standard Mode** (no EDNS): Max 512 bytes
- **EDNS(0) Mode**: Max 4096 bytes (or client-specified size)

Truncated responses:

- Set the `TC` (Truncated) flag in the header
- Keep the question section intact
- Remove all answer, authority, and additional records
- Signal to the client to retry over TCP for the complete response

## Development

### Project Structure Notes

- `common/`: Shared data structures (no business logic)
- `decoder/`: Only handles binary → struct conversion
- `encoder/`: Only handles struct → binary conversion
- `storage/`: Manages record persistence and retrieval
- `server.rs`: Orchestrates request/response cycle

## Next features in the pipes

- [x] EDNS(0) support
- [x] TCP support (RFC 1035 compliant with 2-byte length framing)
- [ ] Additional record type implementations (MX, NS, SOA, PTR, etc.)
- [ ] TTL-based cache expiration

## License

This project is licensed under the Creative Commons Attribution-NonCommercial 4.0 International License - see the [LICENSE](LICENSE) file for details.

**You may NOT use this software for commercial purposes.**

## References

- [RFC 1035 - Domain Names - Implementation and Specification](https://datatracker.ietf.org/doc/html/rfc1035)
- [RFC 3596 - DNS Extensions to Support IPv6 (AAAA records)](https://datatracker.ietf.org/doc/html/rfc3596)
- [RFC 6891 - Extension Mechanisms for DNS (EDNS)](https://datatracker.ietf.org/doc/html/rfc6891)
