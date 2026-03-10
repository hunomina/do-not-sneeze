use std::{
    io::Result,
    net::{SocketAddr, TcpStream, ToSocketAddrs, UdpSocket},
};

pub struct UdpClient {}

impl UdpClient {
    pub fn request<A: ToSocketAddrs>(
        addr: A,
        body: &[u8],
        buffer: &mut [u8],
    ) -> Result<(usize, SocketAddr)> {
        UdpSocket::bind("0.0.0.0:0").and_then(|socket| {
            socket.connect(addr)?;
            socket.send(body)?;
            socket.recv_from(buffer)
        })
    }
}

pub struct TcpClient {}

impl TcpClient {
    pub fn request<A: ToSocketAddrs>(addr: A, body: &[u8]) -> Result<Vec<u8>> {
        use std::io::Write;

        let mut stream = TcpStream::connect(addr)?;

        stream.write_all(
            {
                let mut v = Vec::with_capacity(2 + body.len());
                v.extend_from_slice(&get_query_size_bytes(body.len())?);
                v.extend_from_slice(body);
                v
            }
            .as_slice(),
        )?;

        Self::read_message_from_stream(&mut stream)
    }

    pub fn read_message_from_stream(stream: &mut TcpStream) -> Result<Vec<u8>> {
        use std::io::Read;

        let message_size = {
            let mut size_buf = [0; 2];
            stream.read_exact(&mut size_buf)?;
            u16::from_be_bytes(size_buf) as usize
        };

        let mut buffer = vec![0u8; message_size];
        stream.read_exact(&mut buffer)?;

        Ok(buffer)
    }
}

fn get_query_size_bytes(query_buffer_size: usize) -> Result<[u8; 2]> {
    if query_buffer_size > u16::MAX as usize {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "body too large: exceeds u16 max size",
        ));
    }

    Ok([
        ((query_buffer_size >> 8) & 0xFF) as u8,
        (query_buffer_size & 0xFF) as u8,
    ])
}
