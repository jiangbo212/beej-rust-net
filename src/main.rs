use std::net::ToSocketAddrs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpSocket;
use tracing::{info, instrument};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    info!("start tcp http server simple");
    tokio::spawn(tcp_http_server_simple());
    info!("start tcp http client simple!");
    tcp_http_client_simple().await;
    loop {

    }
}

/// 简单的从tcp开始的http client实现。
/// 注意hyper text content最后的一行空白行一定要保留，且每行的开始不能有留白。
#[instrument]
async fn tcp_http_client_simple() {
    let hyper_text_client_content = br#"
GET / HTTP/1.1
Host: example.com
Connection: close

    "#;
    // 获取远程服务器的socket地址(ipv4或ipv6, 内部通过dns解析获取)
    let addrs = "127.0.0.1:8080".to_socket_addrs().unwrap();
    for addr in addrs {
        info!("socket_addr:{:?}", &addr);
        // 开启本地socket端口
        let socket = TcpSocket::new_v4().unwrap();
        // 连接到远程socket端口
        let mut connect = socket.connect(addr).await.unwrap();
        // 传输数据到远程socket
        connect.write(hyper_text_client_content).await.unwrap();
        let mut buffer: [u8;4096] = [0; 4096];
        // 读取远程socket返回的数据
        let n = connect.read(&mut buffer).await.unwrap();
        info!("example return byte size:{}", n);
        let hyper_text_server_content = String::from_utf8(buffer.to_vec()).unwrap();
        info!("example return content:{}", hyper_text_server_content);
    }
}

#[instrument]
async fn tcp_http_server_simple() {
    let addr = "127.0.0.1:8080".parse().unwrap();
    let socket = TcpSocket::new_v4().unwrap();
    socket.set_reuseaddr(true).unwrap();
    socket.bind(addr).unwrap();
    let tcp_listener = socket.listen(1024).unwrap();
    loop {
        match tcp_listener.accept().await {
            Ok((mut tcp_stream, socket_addr)) => {
                info!("server socket:{:?}", socket_addr);
                let mut buffer: [u8;1024] = [0; 1024];
                tcp_stream.read(&mut buffer).await.unwrap();
                let read_content = String::from_utf8(buffer.to_vec()).unwrap();
                info!("read content: {}", read_content);
                tcp_stream.write(b"Hello World").await.unwrap();
            },
            Err(e) => info!("accept data error, {:?}", e),
        }
    }
}