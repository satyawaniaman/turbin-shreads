use tokio::net::UdpSocket;

pub struct UdpShredListener {
    socket: UdpSocket,
}

const MAX_SHRED_SIZE: usize = 1228;

impl UdpShredListener {
    pub async fn bind(addr: &str) -> anyhow::Result<Self> {
        let socket = UdpSocket::bind(addr).await?;
        tracing::info!(addr, "UDP shred listener bound");
        Ok(Self { socket })
    }

    pub async fn recv(&self) -> anyhow::Result<Vec<u8>> {
        let mut buf = vec![0u8; MAX_SHRED_SIZE];
        let (len, _addr) = self.socket.recv_from(&mut buf).await?;
        buf.truncate(len);
        Ok(buf)
    }

    pub fn local_addr(&self) -> anyhow::Result<std::net::SocketAddr> {
        Ok(self.socket.local_addr()?)
    }
}
