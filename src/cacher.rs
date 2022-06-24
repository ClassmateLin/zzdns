use std::{sync::Arc, time::Duration};

use anyhow::{Result};
use bytes::{Bytes, BytesMut};
use domain::base::{Message, MessageBuilder, iana::Rcode};
use stretto::AsyncCache;

pub struct Cacher {}

impl Cacher {

    // 缓存DNS报文
    pub(crate) async fn set_response_msg(qmsg: Message<Bytes>, rmsg: Message<Bytes>, cache:Arc<AsyncCache<String, Bytes>>) -> Result<bool> {
        let question = qmsg.sole_question()?;
        let qname = question.qname();
        let (_, ans,_,_) = rmsg.sections()?;
        let mut rmsg = MessageBuilder::from_target(BytesMut::with_capacity(1024))?
        .start_answer(&qmsg, Rcode::NoError)?;
        let header = rmsg.header_mut();
        header.set_ra(true);
        let mut ttl = u32::MAX;
        for item in ans {
            
            if let Ok(rr) = item {
                if rr.ttl() < ttl {
                    ttl = rr.ttl();
                }
                if rr.rtype()  !=  domain::base::Rtype::A && rr.rtype() != domain::base::Rtype::Cname {
                    continue;
                }
                if let Ok(record) = rr.to_record::<domain::rdata::rfc1035::A>() {
                    if !record.is_none() {
                        let record = record.unwrap();
                        rmsg.push(record).unwrap();
                    }
                }
                if let Ok(record) = rr.to_record::<domain::rdata::rfc1035::Cname<_>>() {
                    if record.is_none() {
                        continue;
                    }
                    let record = record.unwrap();
                    rmsg.push(record).unwrap();
                    
                }
            };
        }

        let flag = cache.insert_with_ttl(qname.to_string(), rmsg.into_message().as_octets().clone(),1 , Duration::from_secs(ttl.into())).await;
        Ok(flag)
    }

    /// 读取缓存的DNS报文和ttl时间
    pub(crate) fn get_response_msg(qmsg: Message<Bytes>, cache:Arc<AsyncCache<String, Bytes>>) -> Result<Option<(Message<Bytes>, u64)>> {
        
        let question = qmsg.sole_question()?;
        let qname = question.qname();
        if let Some(val) = cache.get(&qname.to_string()) {
            let buf = val.value();
            let ttl =  val.ttl();

            let mut bytes = BytesMut::with_capacity(1024);
            bytes.resize(buf.len(), 0);
            bytes.copy_from_slice(&buf);

            let id:u16 = qmsg.header().id();

            bytes[0] = ((id >> 8) & 0xff) as u8;
            bytes[1] = (id & 0xff) as u8;

            let rmsg:Message<Bytes> =  Message::from_octets(bytes.freeze())?;
            
            return Ok(Option::Some((rmsg, ttl.as_secs())));
        };      
        
        Ok(None)
    }
}