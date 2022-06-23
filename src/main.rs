use tokio::{net::UdpSocket, sync::mpsc, time::{sleep, Duration}, io::AsyncReadExt};
use std::{io::Result, sync::Arc, net::SocketAddr, str::Bytes};
use fastdns::dns;


async fn process(data: &[u8]) -> Result<Vec<u8>> {
    // println!("raw data:{:?}", data);
    // let (header, header_len) = dns::DNSHeader::parse(&data.to_vec());
    // println!("header:{:?}, len:{:?}", header, header_len);
    // println!("{}", header.qr());
    // println!("{}", header.op_code());
    // println!("{}", header.aa());
    // println!("{}", header.tc());
    // println!("{}", header.rd());
    // println!("{}", header.ra());
    // println!("{}", header.z());
    // println!("{}", header.r_code());
    // let (question, question_len) =  dns::DNSQuestion::parse(data[header_len..].to_vec());
    // println!("q name:{:?}, len:{}",  question.q_name(), question_len);
    let mut buf = [0;512];
    let sock = UdpSocket::bind("0.0.0.0:29999".parse::<SocketAddr>().unwrap()).await?;
    let server_addr = "8.8.8.8:53".parse::<SocketAddr>().unwrap();
    println!("raw:");
    let len = sock.send_to(data, server_addr).await?;
    let (len, addr) = sock.recv_from(&mut buf).await?;
    let (header, header_len) = dns::DNSHeader::parse(&data.to_vec());
    let (question, question_len) =  dns::DNSQuestion::parse(data[header_len..].to_vec());
    println!("{}", question.q_name());
    dns::DNSResourceRecord::parse(data[header_len+question_len..].to_vec());
    Ok(buf.to_vec())
}


async fn start_server(port: u16) -> Result<()> {

    let server_addr = format!("0.0.0.0:{port}").parse::<SocketAddr>().unwrap();
    let sock = UdpSocket::bind(server_addr).await.unwrap();
    let reader = Arc::new(sock);
    let sender = reader.clone();

    let (tx, mut rx) = mpsc::channel::<(Vec<u8>, SocketAddr)>(1_000);
    
    let (tx2, mut rx2) = mpsc::channel::<(Vec<u8>, SocketAddr)>(3);

    tokio::spawn(async move { // process data
        while let Some((bytes, addr)) = rx.recv().await {
            let tx2_clone = tx2.clone();
            tokio::spawn(async move {
                let data = process(&bytes).await.unwrap();
                tx2_clone.send((data, addr)).await.unwrap();
            });
        }
    });

    tokio::spawn(async move { // send data
        while let Some((bytes, addr)) = rx2.recv().await {
            let _len = sender.send_to(&bytes, &addr).await.unwrap();
        }
    });

    let mut buf = [0; 512];
    loop { // receive data
        let (len, addr) = reader.recv_from(&mut buf).await?;
        tx.send((buf[..len].to_vec(), addr)).await.unwrap();
    }
}

#[tokio::main]
async fn main() -> Result<()>{
    start_server(9999).await?;
    Ok(())
}