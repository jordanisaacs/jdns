use std::{net::SocketAddr, sync::Arc};

use bytes::Bytes;
use tokio::{
    net::UdpSocket,
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
};

pub type TxUdp = UnboundedSender<(Bytes, SocketAddr)>;
pub type RxUdp = UnboundedReceiver<(Bytes, SocketAddr)>;

pub struct SendPool {
    tx: TxUdp,
}

impl SendPool {
    pub fn new(udp_socket: Arc<UdpSocket>) -> Self {
        let (tx, rx) = unbounded_channel();
        Self::recv(rx, udp_socket);
        SendPool { tx }
    }

    pub fn get_tx(&self) -> TxUdp {
        self.tx.clone()
    }

    fn recv(mut rx: RxUdp, udp_socket: Arc<UdpSocket>) {
        tokio::spawn(async move {
            while let Some((data, addr)) = rx.recv().await {
                println!("Sending to addr: {:?}", addr);
                let _ = udp_socket.send_to(&data, &addr).await;
            }
        });
    }
}
