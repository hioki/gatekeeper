use std::io;
use std::net::TcpStream;

use crate::byte_stream::ByteStream;
use crate::pkt_stream::{PktStream, UdpPktStream};

use model::error::{Error, ErrorKind};
use model::model::*;

use failure::Fail;

pub trait Connector: Send {
    type B: ByteStream;
    type P: PktStream;
    fn connect_byte_stream(&self, addr: Address) -> Result<Self::B, Error>;
    fn connect_pkt_stream(&self, addr: Address) -> Result<Self::P, Error>;
}

#[derive(Debug, Clone)]
pub struct TcpUdpConnector;

impl Connector for TcpUdpConnector {
    type B = TcpStream;
    type P = UdpPktStream;
    fn connect_byte_stream(&self, addr: Address) -> Result<Self::B, Error> {
        match &addr {
            Address::IpAddr(addr, port) => TcpStream::connect(SocketAddr::new(*addr, *port)),
            Address::Domain(host, port) => TcpStream::connect((host.as_str(), *port)),
        }
        .map_err(|err| {
            match err.kind() {
                io::ErrorKind::ConnectionRefused => {
                    ErrorKind::connection_refused(addr, L4Protocol::Tcp).into()
                }
                _ => err.context(ErrorKind::Io),
            }
            .into()
        })
    }
    fn connect_pkt_stream(&self, _addr: Address) -> Result<Self::P, Error> {
        unimplemented!("connect_pkt_stream")
        /*
        let sock_addr = self.resolve(addr)?;
        UdpSocket::connect(sock_addr).map_err(Into::into)
        */
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::byte_stream::ByteStream;
    use model::ErrorKind;
    use std::collections::BTreeMap;

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    pub struct BufferConnector<S> {
        pub strms: BTreeMap<Address, Result<S, ConnectError>>,
    }

    impl<S> BufferConnector<S> {
        pub fn stream(&self, addr: &Address) -> &S {
            &self.strms[addr].as_ref().unwrap()
        }
    }

    impl<S> Connector for BufferConnector<S>
    where
        S: ByteStream + Clone,
    {
        type B = S;
        type P = UdpPktStream;
        fn connect_byte_stream(&self, addr: Address) -> Result<Self::B, Error> {
            println!("connect_byte_stream: {:?}", &addr);
            match &self.strms[&addr] {
                Ok(strm) => Ok(strm.clone()),
                Err(err) => {
                    use Address::*;
                    use ConnectError::*;
                    use L4Protocol::*;
                    let kind = match err {
                        NetworkUnreachable => match addr {
                            Domain(domain, port) => ErrorKind::DomainNotResolved { domain, port },
                            IpAddr(ipaddr, port) => ErrorKind::HostUnreachable {
                                host: ipaddr.to_string(),
                                port,
                            },
                        },
                        HostUnreachable => {
                            let port = addr.port();
                            let host = match addr {
                                Domain(domain, _) => domain,
                                IpAddr(ipaddr, _) => ipaddr.to_string(),
                            };
                            ErrorKind::HostUnreachable { host, port }
                        }
                        ConnectionNotAllowed => ErrorKind::connection_not_allowed(addr, Tcp),
                        ConnectionRefused => ErrorKind::connection_refused(addr, Tcp),
                        _ => ErrorKind::Io,
                    };
                    Err(kind.into())
                }
            }
        }
        fn connect_pkt_stream(&self, _addr: Address) -> Result<Self::P, Error> {
            unimplemented!("BufferConnector::connect_pkt_stream")
        }
    }
}
