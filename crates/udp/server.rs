use std::{
    error::Error,
    future::Future,
    io,
    marker::PhantomData,
    net::{SocketAddr, ToSocketAddrs},
    sync::Arc,
};

use super::send::{SendPool, TxUdp};
use bytes::Bytes;
use futures::future::join_all;
use socket2::{Domain, Socket, Type};
use tokio::net::UdpSocket;

pub const BUFF_MAX_SIZE: usize = 4096;

pub struct UdpServer<H, R, S> {
    threads: Vec<UdpThread>,
    handle: Arc<H>,
    state: S,
    handle_return: PhantomData<R>,
}

pub struct UdpThread {
    pub send: SendPool,
    pub sock: Arc<UdpSocket>,
}

/// Resolve a ToSocketAddrs only if it resolves to a single address
/// From [net2](https://github.com/deprecrated/net2-rs/blob/a18347549413975fbbeb5567165f163e5f60a627/src/lib.rs#L77)
fn one_addr<T: ToSocketAddrs>(tsa: T) -> io::Result<std::net::SocketAddr> {
    let mut addrs = tsa.to_socket_addrs()?;
    let addr = match addrs.next() {
        Some(addr) => addr,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "no socket addresses could be resolved",
            ))
        }
    };
    if addrs.next().is_none() {
        Ok(addr)
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "more than one address resolved",
        ))
    }
}

impl<H, R, S> UdpServer<H, R, S>
where
    H: Fn(SocketAddr, Bytes, S, TxUdp) -> R + Send + Sync + 'static,
    R: Future<Output = ()> + Send,
    S: Send + Sync + Clone + 'static,
{
    /// Create a udp socket
    fn create_udp_socket<A: ToSocketAddrs>(addr: &A) -> Result<UdpSocket, Box<dyn Error>> {
        let sock = Socket::new(Domain::IPV4, Type::DGRAM, None)?;
        sock.set_reuse_port(true)?;
        sock.set_reuse_address(true)?;

        let single_addr = one_addr(addr)?;
        sock.bind(&single_addr.into())?;

        sock.set_send_buffer_size(512 * 1500)?;
        sock.set_recv_buffer_size(512 * 1500)?;

        sock.set_nonblocking(true)?;
        let udp_sock = UdpSocket::from_std(sock.into())?;
        Ok(udp_sock.into())
    }

    /// Create a list of udp sockets
    fn create_udp_socket_list<A: ToSocketAddrs>(
        addr: &A,
        listen_count: usize,
    ) -> Result<Vec<UdpSocket>, Box<dyn Error>> {
        let mut socks = Vec::with_capacity(listen_count);
        for _ in 0..listen_count {
            let sock = Self::create_udp_socket(addr)?;
            socks.push(sock);
        }
        Ok(socks)
    }

    pub fn new<A: ToSocketAddrs>(
        addr: A,
        handle: H,
        init_state: S,
    ) -> Result<Self, Box<dyn Error>> {
        let num_cpus = num_cpus::get();
        let sock_list = Self::create_udp_socket_list(&addr, num_cpus)?;

        let mut threads = Vec::with_capacity(num_cpus);
        for sock in sock_list.into_iter() {
            let arc_sock = Arc::new(sock);

            threads.push(UdpThread {
                send: SendPool::new(arc_sock.clone()),
                sock: arc_sock,
            })
        }

        Ok(Self {
            threads,
            handle: Arc::new(handle),
            state: init_state,
            handle_return: PhantomData::default(),
        })
    }

    pub async fn start(&self) -> Result<(), Box<dyn Error>> {
        let mut join_handles = Vec::with_capacity(self.threads.len());
        for udp_thread in &self.threads {
            let state = self.state.clone();
            let handle = self.handle.clone();

            let move_sock = udp_thread.sock.clone();
            let tx = udp_thread.send.get_tx().clone();

            let join_handle = tokio::spawn(async move {
                let mut buff = [0; BUFF_MAX_SIZE];
                loop {
                    match move_sock.recv_from(&mut buff).await {
                        Ok((size, addr)) => {
                            let tx = tx.clone();
                            let state = state.clone();
                            let handle = handle.clone();

                            tokio::spawn(async move {
                                handle(addr, Bytes::copy_from_slice(&buff[..size]), state, tx)
                                    .await;
                            });
                        }
                        Err(er) => {
                            println!("Recieve error: {:?}", er);
                        }
                    }
                }
            });

            join_handles.push(join_handle);
        }

        println!(
            "Starting UDP server on: {:?}",
            self.threads[0].sock.local_addr()
        );
        join_all(join_handles).await;

        Ok(())
    }
}
