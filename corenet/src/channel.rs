use std::{
    io::{Read, Result as IoResult, Write},
    net::{Shutdown, TcpStream},
};

#[derive(Debug)]
pub struct TcpChannel {
    stream: TcpStream,
}

impl TcpChannel {
    pub fn new(stream: TcpStream) -> Self {
        Self { stream }
    }

    pub fn shutdown(&self) -> IoResult<()> {
        self.stream.shutdown(Shutdown::Both)
    }

    pub fn peer_addr(&self) -> IoResult<std::net::SocketAddr> {
        self.stream.peer_addr()
    }
}

impl Read for TcpChannel {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        self.stream.read(buf)
    }
}

impl Write for TcpChannel {
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        self.stream.write(buf)
    }

    fn flush(&mut self) -> IoResult<()> {
        self.stream.flush()
    }
}
