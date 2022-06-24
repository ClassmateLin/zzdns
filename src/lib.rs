use std::{sync::Arc};

#[macro_use]extern crate lazy_static;
#[macro_use] extern crate log;

extern crate tokio;
extern crate pretty_env_logger;
use anyhow::Result;
use bytes::Bytes;
use config::{get_bind_addr, get_cache_max_size};
use cache_server::start_cache_server;
use dns_server::start_dns_server;
use domain::base::Message;
use stretto::AsyncCache;
use tokio::{net::UdpSocket, sync::{broadcast, mpsc}};

mod upstream;
mod resolver;
mod config;
mod cache_server;
mod dns_server;
mod cacher;
mod prefetcher;


pub async fn init_servers() -> Result<()>{
    
    let socket = UdpSocket::bind(get_bind_addr()).await.expect("Create a failed to create a socket, unable to monitor 0.0.0.0:3053, please check whether the port is occupied by other services.");
    let socket = Arc::new(socket);
    
    let (tx, _) = broadcast::channel::<()>(10);
    let tx = Arc::new(tx);
    
    let cache:AsyncCache<String, Bytes> = AsyncCache::new(get_cache_max_size() as usize, 1e6 as i64,
         tokio::spawn).unwrap();
    let cache = Arc::new(cache);
    
    let dns_server_tx = tx.clone();
    let cache_server_tx = tx.clone();

    let (msg_sender, msg_receiver) = mpsc::channel::<(Message<Bytes>, Message<Bytes>)>(1000);
    let msg_sender = Arc::new(msg_sender);

    let server_cache = cache.clone();
    let preloader_cache = cache.clone();
    
    start_cache_server(cache_server_tx, preloader_cache.clone(), msg_receiver).await;
    prefetcher::start_prefetcher(msg_sender.clone()).await;
    start_dns_server(socket, dns_server_tx, server_cache, msg_sender.clone()).await
    
}