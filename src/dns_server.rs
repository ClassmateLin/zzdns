use std::{sync::Arc, net::SocketAddr, time::Duration};
use bytes::{BytesMut, Bytes};
use anyhow::{Result};
use stretto::AsyncCache;
use tokio::{net::UdpSocket, sync::{broadcast::Sender, mpsc}, signal, time::sleep};
use domain::base::{iana::rcode::Rcode, Message, MessageBuilder};
use crate::{resolver::Resolver, config::get_bind_addr};
pub struct DnsServer {}

impl DnsServer {
    
    async fn route(qmsg: Message<Bytes>, cache:Arc<AsyncCache<String, Bytes>>, msg_sender: Arc<mpsc::Sender<(Message<Bytes>, Message<Bytes>)>>) -> Result<Message<Bytes>> {
        let question = qmsg.sole_question()?;
        match question.qtype() {
            domain::base::Rtype::A => return Resolver::resolve_a(qmsg, cache.clone(), msg_sender).await,
            _ => return Resolver::resolve_other(qmsg).await,
        };
    }

    /// worker
    async fn worker(
        socket: Arc<UdpSocket>,
        bytes: Bytes,
        src: SocketAddr,
        cache: Arc<AsyncCache<String, Bytes>>,
        msg_sender: Arc<mpsc::Sender<(Message<Bytes>, Message<Bytes>)>>
    ) -> Result<()> {
        let req_msg = Message::from_octets(bytes)?;
        let resp_msg = match req_msg.sole_question(){
            Ok(_) => Self::route(req_msg, cache.clone(), msg_sender).await?,
            Err(e) => {
                warn!("Message parsing error: {}", e);
                MessageBuilder::from_target(BytesMut::with_capacity(1024))?
                .start_answer(&req_msg, Rcode::ServFail)?
                .into_message()
            }
        };
        let _ = socket.send_to(resp_msg.as_slice(), src).await?;
        Ok(())
    }

    /// serve
    async fn serve(socket: Arc<UdpSocket>, tx: Arc<Sender<()>>, cache: Arc<AsyncCache<String, Bytes>>, msg_sender: Arc<mpsc::Sender<(Message<Bytes>, Message<Bytes>)>>) -> Result<()>{
        info!("DNS service running, listen at: {}.", get_bind_addr());
        loop {
            let mut buf = BytesMut::with_capacity(1024);

            buf.resize(1024, 0);

            let (len, src) = match socket.recv_from(&mut buf).await {
                Ok(r) => r,
                Err(e) => {
                    error!("Unable to read data, error:{}", e);
                    continue;
                }
            };

            buf.resize(len, 0);
            let socket = socket.clone();
            let cache = cache.clone();
            let mut shutdown = tx.subscribe();
            let msg_sender = msg_sender.clone();
            tokio::spawn(async move {
                tokio::select! {
                    biased; _ = Self::worker(socket, buf.freeze(), src, cache, msg_sender) => {}
                    _ = shutdown.recv() => {}
                }
            });

        }
    }
}


/// 启动DNS服务
pub async fn start_dns_server(socket: Arc<UdpSocket>, tx: Arc<Sender<()>>, cache: Arc<AsyncCache<String, Bytes>>, msg_sender: Arc<mpsc::Sender<(Message<Bytes>, Message<Bytes>)>>) -> Result<()> {
    #[rustfmt::skip]
    tokio::select! {
        _ = DnsServer::serve(socket, tx.clone(), cache.clone(), msg_sender) => (),
        
        _ = signal::ctrl_c() => {
            info!("Stop DNS service...");
            let tx = tx.clone();
	        sleep(Duration::from_millis(500)).await;
            if tx.send(()).is_ok() {
                while tx.receiver_count() != 0 {
                    sleep(Duration::from_secs(1)).await
                }
            }
        }
    };
    Ok(())
}


