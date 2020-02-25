use std::net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs};

use log::*;

use crate::byte_stream::ByteStream;
use crate::error::Error;

pub struct TcpAcceptor {
    listener: TcpListener,
}

impl Iterator for TcpAcceptor {
    type Item = (TcpStream, SocketAddr);
    fn next(&mut self) -> Option<Self::Item> {
        match self.listener.accept() {
            Ok(x) => Some(x),
            Err(err) => {
                error!("accept error: {}", err);
                trace!("accept error: {:?}", err);
                None
            }
        }
    }
}

pub trait Binder {
    type Stream: ByteStream + Send + 'static;
    type Iter: Iterator<Item = (Self::Stream, SocketAddr)> + Send + 'static;
    fn bind<A: ToSocketAddrs>(&self, addr: A) -> Result<Self::Iter, Error>;
}

pub struct TcpBinder;

impl Binder for TcpBinder {
    type Stream = TcpStream;
    type Iter = TcpAcceptor;
    fn bind<A: ToSocketAddrs>(&self, addr: A) -> Result<Self::Iter, Error> {
        Ok(TcpAcceptor {
            listener: TcpListener::bind(addr)?,
        })
    }
}