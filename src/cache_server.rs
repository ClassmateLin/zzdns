/// DNS缓存服务器
use std::sync::Arc;
use bytes::Bytes;
use stretto::AsyncCache;
use tokio::time::{Duration, sleep};
use anyhow::Result;
use tokio::{sync::broadcast::Sender, signal};
use tokio::sync::mpsc;
use crate::cacher::Cacher;
use domain::base::Message;

pub struct CacheServer {}

/// 缓存服务器
/// 监听消息队列, 收到DNS报文后, 解析A记录, 进行测速, 缓存。
impl CacheServer {

    async fn handle(cache: Arc<AsyncCache<String, Bytes>>, qmsg: Message<Bytes>, rmsg: Message<Bytes>) -> Result<()>{
        // if let Some(ttl) = cache.get_ttl(&q_name) {
        //     if ttl > Duration::from_secs(5) {
        //         return Ok(());
        //     }
        // };
        // let dname = Dname::bytes_from_str(&q_name).unwrap();
        // let qtype = Rtype::A;
        // let req_up_msg = MessageBuilder::from_target(BytesMut::with_capacity(1024))?;
        // let mut question_builder = req_up_msg.question();
        // question_builder.push(Question::new_in(dname.clone(), qtype)).unwrap();
        // let up_msg = Upstream::query(question_builder.clone().into_message()).await?;
        // let _ = cacher::Cacher::set_response_msg(question_builder.clone().into_message(), up_msg.clone(), cache.clone()).await;
        // Ok(())
        let flag = Cacher::set_response_msg(qmsg.clone(), rmsg, cache).await?;
        if flag {
            let question = qmsg.sole_question()?;
            info!("success cache {}...", question.qname());
        }
        Ok(())
    }

    /// 处理
    async fn worker(cache: Arc<AsyncCache<String, Bytes>>, mut msg_receiver: mpsc::Receiver<(Message<Bytes>, Message<Bytes>)>) {
        info!("Start the cache server.");
        loop {
            tokio::select! {
                res = msg_receiver.recv() => {
                    if let Some((qmsg, rmsg)) = res {
                        let _ = Self::handle(cache.clone(), qmsg, rmsg).await;
                    };
                }
                _ = sleep(Duration::from_secs(1)) => {},
            }
        }
    }

    async fn serve(cache: Arc<AsyncCache<String, Bytes>>, tx: Arc<Sender<()>>, msg_receiver: mpsc::Receiver<(Message<Bytes>, Message<Bytes>)>) {
        
        tokio::select! {
            _ = Self::worker(cache, msg_receiver) => {}
            _ = signal::ctrl_c() => {
                info!("Stop DNS preloading...");
                let tx = tx.clone();
                sleep(Duration::from_millis(500)).await;
                if tx.send(()).is_ok() {
                    while tx.receiver_count() != 0 {
                        sleep(Duration::from_secs(1)).await
                    }
                }
            }
        }
    }
}


// 启动缓存服务
pub async fn start_cache_server(tx: Arc<Sender<()>>, cache: Arc<AsyncCache<String, Bytes>>, msg_receiver: mpsc::Receiver<(Message<Bytes>, Message<Bytes>)>) {
    tokio::spawn(async move {
        CacheServer::serve(cache, tx, msg_receiver).await;
    });
}