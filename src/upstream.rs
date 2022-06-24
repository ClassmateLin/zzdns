use bytes::{Bytes, BytesMut};

use tokio::{net::UdpSocket, sync::mpsc, sync::mpsc::Sender, time::sleep};

use std::{sync::Arc, net::SocketAddr, time::Duration};
use anyhow::{Result, Ok};
use crate::config::get_upstream_dns_addrs;
use domain::base::{iana::rcode::Rcode, Message, MessageBuilder};
pub struct Upstream {}


impl Upstream {

    async fn request(socket: Arc<UdpSocket>,tx: Sender<Bytes>, dns_query_buf: Vec<u8>, server_addr: SocketAddr) -> Result<()> {
        
        let mut buf = BytesMut::with_capacity(1024);
        buf.resize(1024, 0);
    
        let _ = socket.send_to(&dns_query_buf, server_addr).await?;
    
        let (len, _addr) = socket.recv_from(&mut buf).await?;
        buf.resize(len, 0);


        tokio::select! {
            _ = tx.closed() => {}
            _ = tx.send_timeout(buf.freeze(), Duration::from_millis(100)) => {}
        }
        Ok(())
    }
    
    pub(crate) async fn query(qmsg: Message<Bytes>) -> Result<Message<Bytes>>{
        
        let client_socket = UdpSocket::bind("0.0.0.0:0".parse::<SocketAddr>().unwrap()).await?;
        let client_socket = Arc::new(client_socket);

        
        let (tx, mut rx) = mpsc::channel::<Bytes>(1);
    
        for server in get_upstream_dns_addrs() {
            tokio::spawn(Self::request(client_socket.clone(), tx.clone(), qmsg.as_slice().to_vec(), server));
        }
    
        tokio::select! {
            buf = rx.recv() => {
                if let Some(buf) = buf {
                    rx.close();
                    return Ok(Message::from_octets(buf).unwrap());
                }else{
                    return Ok(MessageBuilder::from_target(BytesMut::with_capacity(1024))?
                    .start_answer(&qmsg, Rcode::ServFail)?
                    .into_message()); 
                }
            },
            _ = sleep(Duration::from_secs(1)) => {
                return Ok(MessageBuilder::from_target(BytesMut::with_capacity(1024))?
                .start_answer(&qmsg, Rcode::Refused)?
                .into_message());
            },
        };
        
    }
}