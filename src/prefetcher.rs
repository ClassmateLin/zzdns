use tokio::time;
use std::{sync::Arc, time::Duration};
use bytes::{Bytes, BytesMut};
use domain::base::{Message, Dname, MessageBuilder, Question, Rtype};
use tokio::{sync::mpsc, fs};
use anyhow::Result;
use crate::{config::{get_preloader_filename, get_preload_domain_count}, upstream::Upstream};


// 读取域名配置列表, 请求DNS域名列表, 将结果发送到缓存服务器进行缓存.
async fn handle(msg_sender: Arc<mpsc::Sender<(Message<Bytes>, Message<Bytes>)>>) -> Result<()> {
    let domain = fs::read_to_string(get_preloader_filename()).await.unwrap();
    let mut count = 0;
    for q_name in domain.lines() {
        
        if count >= get_preload_domain_count() {
            break;
        }

        let qname = Dname::bytes_from_str(&q_name).unwrap();
        let qmsg_builder = MessageBuilder::from_target(BytesMut::with_capacity(1024))?;
        
        let mut question_builder = qmsg_builder.question();
        question_builder.push(Question::new_in(qname.clone(), Rtype::A)).unwrap();
        
        let qmsg = question_builder.into_message();
      
        if let Ok(rmsg) = Upstream::query(qmsg.clone()).await { 
            match msg_sender.send((qmsg, rmsg)).await {
                Ok(_) => count += 1,
                Err(err) => error!("DNS packets of the domain({}) cannot be sent to the cache server. error:{:?}", qname, err.to_string()),
            }
        };
    }
    Ok(())
}


/// 启动域名预加载
pub(crate) async fn start_prefetcher(msg_sender: Arc<mpsc::Sender<(Message<Bytes>, Message<Bytes>)>>) {
    tokio::spawn(async move {
        info!("The domain name pre-resolution service is running.");
        let _ = handle(msg_sender).await;
        time::sleep(Duration::from_secs(2)).await;
        info!("The domain name pre-resolution service has been closed.");
    });
}

