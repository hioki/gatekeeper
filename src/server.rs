use std::net::ToSocketAddrs;
use std::ops::Deref;
use std::sync::mpsc::{self, SyncSender};
use std::thread;

use log::*;

use crate::acceptor::Binder;
use crate::error::Error;
use crate::server_command::ServerCommand;

pub struct Server<T> {
    tx_cmd: mpsc::SyncSender<ServerCommand>,
    rx_cmd: mpsc::Receiver<ServerCommand>,
    binder: T,
}

fn spawn_acceptor(
    binder: impl Deref<Target = impl Binder>,
    tx: SyncSender<ServerCommand>,
    addr: impl ToSocketAddrs,
) -> Result<thread::JoinHandle<()>, Error> {
    let acceptor = binder.bind(addr)?;
    Ok(thread::spawn(move || {
        use ServerCommand::*;
        for (strm, addr) in acceptor {
            info!("accept: {}", addr);
            if tx.send(Connect(Box::new(strm), addr)).is_err() {
                info!("disconnected ServerCommand chan");
                break;
            }
        }
    }))
}

impl<T> Server<T>
where
    T: Binder,
{
    pub fn new(binder: T) -> (Self, mpsc::SyncSender<ServerCommand>) {
        let (tx, rx) = mpsc::sync_channel(0);
        (
            Self {
                tx_cmd: tx.clone(),
                rx_cmd: rx,
                binder,
            },
            tx,
        )
    }

    pub fn serve(&self) -> Result<(), Error> {
        spawn_acceptor(&self.binder, self.tx_cmd.clone(), "127.0.0.1:1080")?;

        while let Ok(cmd) = self.rx_cmd.recv() {
            use ServerCommand::*;
            debug!("cmd: {:?}", cmd);
            match cmd {
                Terminate => break,
                Connect(_stream, _addr) => {}
            }
        }
        info!("server shutdown");
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::acceptor::{Binder, TcpBinder};
    use crate::byte_stream::test::*;

    use std::borrow::Cow;
    use std::net::*;
    use std::ops::Deref;
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, SystemTime};

    #[test]
    fn server_shutdown() {
        let (server, tx) = Server::new(TcpBinder);
        let shutdown = Arc::new(Mutex::new(SystemTime::now()));
        let th = {
            let shutdown = shutdown.clone();
            thread::spawn(move || {
                server.serve().ok();
                *shutdown.lock().unwrap() = SystemTime::now();
            })
        };
        thread::sleep(Duration::from_secs(1));
        let req_shutdown = SystemTime::now();
        tx.send(ServerCommand::Terminate).unwrap();
        th.join().unwrap();
        assert!(shutdown.lock().unwrap().deref() > &req_shutdown);
    }

    struct DummyBinder {
        stream: BufferStream,
        src_addr: SocketAddr,
    }

    impl Binder for DummyBinder {
        type Stream = BufferStream;
        type Iter = std::iter::Once<(Self::Stream, SocketAddr)>;
        fn bind<A: ToSocketAddrs>(&self, addr: A) -> Result<Self::Iter, Error> {
            let mut addr = addr.to_socket_addrs().unwrap();
            println!("bind: {}", addr.next().unwrap());
            Ok(std::iter::once((self.stream.clone(), self.src_addr)))
        }
    }

    #[test]
    fn dummy_binder() {
        let binder = DummyBinder {
            stream: BufferStream::new(Cow::from(b"dummy".to_vec())),
            src_addr: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 1080)),
        };
        let (server, tx) = Server::new(binder);
        let th = thread::spawn(move || {
            server.serve().ok();
        });

        thread::sleep(Duration::from_secs(1));
        tx.send(ServerCommand::Terminate).unwrap();
        th.join().unwrap();
    }
}
