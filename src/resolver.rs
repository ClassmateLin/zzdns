use std::sync::Arc;
use domain::base::{iana::Rcode};
use bytes::{Bytes, BytesMut};
use domain::base::{Message, MessageBuilder, Question};
use anyhow::{Result};
use stretto::AsyncCache;
use tokio::sync::mpsc;
use crate::upstream::Upstream;
use domain::base::name::Dname;
use crate::cacher::Cacher;
pub struct Resolver {}

impl Resolver {

    /// 处理a记录
    pub(crate) async fn resolve_a(qmsg: Message<Bytes>, cache:Arc<AsyncCache<String, Bytes>>, msg_sender: Arc<mpsc::Sender<(Message<Bytes>, Message<Bytes>)>>) -> Result<Message<Bytes>> {
        let question = qmsg.sole_question()?;
        let qname = question.qname();
        let qtype = question.qtype();
        let dname = Dname::bytes_from_str(&qname.to_string()).unwrap();
        let mut rmsg = MessageBuilder::from_target(BytesMut::with_capacity(1024))?
                .start_answer(&qmsg, Rcode::NoError)?;
        let header = rmsg.header_mut();
        header.set_ra(true);

        // 有缓存则返回返回结果
        if let Ok(Some((rmsg, ttl))) = Cacher::get_response_msg(qmsg.clone(), cache.clone()){
            let qmsg2 = qmsg.clone();
                let (_, answer, _, _) = rmsg.sections().unwrap();
                let mut rmsg = MessageBuilder::from_target(BytesMut::with_capacity(1024))?
                    .start_answer(&qmsg2, Rcode::NoError).unwrap();
                
                // 更新Msg的缓存为剩余的ttl.
                for rr in answer.flatten() {
            
                    if rr.rtype()  !=  domain::base::Rtype::A && rr.rtype() != domain::base::Rtype::Cname {
                        continue;
                    }
                    
                if let Ok(record) = rr.to_record::<domain::rdata::rfc1035::A>() {
                    if record.is_some() {
                        let mut record = record.unwrap();
                        record.set_ttl(ttl as u32);
                        rmsg.push(record).unwrap();
                    }
                }
                if let Ok(record) = rr.to_record::<domain::rdata::rfc1035::Cname<_>>() {
                    if record.is_none() {
                        continue;
                    }
                    let mut record = record.unwrap();
                    record.set_ttl(ttl as u32);
                    rmsg.push(record).unwrap();
                        
                }
                }    
                if ttl < 10 { // 重新缓存
                    tokio::spawn(async move {
                        info!("recache...");
                        if let Ok(rmsg) =  Upstream::query(qmsg).await {
                            let _ = msg_sender.send((qmsg2, rmsg)).await;
                        }
                    });
                    return Ok(rmsg.into_message());
                }
                return Ok(rmsg.into_message());
        };

        let req_up_msg = MessageBuilder::from_target(BytesMut::with_capacity(1024))?;
        let mut question_builder = req_up_msg.question();
        question_builder.push(Question::new_in(dname.clone(), qtype)).unwrap();
        let header = question_builder.header_mut();
        header.set_id(qmsg.header().id());

        let up_msg = Upstream::query(question_builder.into_message()).await?;
        let (_, ans,_,_) = up_msg.sections()?;

        
        let mut has_a = false;
        for rr in ans.flatten() {
            if rr.rtype()  !=  domain::base::Rtype::A && rr.rtype() != domain::base::Rtype::Cname {
                continue;
            }
            
            if let Ok(record) = rr.to_record::<domain::rdata::rfc1035::A>() {
                if record.is_some() {
                    let mut record = record.unwrap();
                    record.set_ttl(5);
                    rmsg.push(record).unwrap();
                    has_a = true;
                }
            }
            if let Ok(record) = rr.to_record::<domain::rdata::rfc1035::Cname<_>>() {
                if record.is_none() {
                    continue;
                }
                let mut record = record.unwrap();
                record.set_ttl(5);
                rmsg.push(record).unwrap();
                
            }
        }
        if has_a {
            let rmsg = rmsg.clone().into_message();
            tokio::spawn(async move {
                let _ = msg_sender.send((qmsg, rmsg)).await;
            });
        }
        Ok(rmsg.into_message())
    }

    /// 处理其他记录
    pub(crate) async fn resolve_other(qmsg: Message<Bytes>) -> Result<Message<Bytes>> {
        Upstream::query(qmsg).await
    }
}

