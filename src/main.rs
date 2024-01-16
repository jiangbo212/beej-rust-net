use std::net::ToSocketAddrs;
use std::ops::Add;
use bytes::{BytesMut};
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
    info!("start connect time");
    tcp_client_time_gov().await;
    loop {

    }
}

/// 简单的从tcp开始的http client实现。
/// 注意hyper text content最后的一行空白行一定要保留，且每行的开始不能有留白。
#[instrument]
async fn tcp_http_client_simple() {
    let hyper_text_client_content = b"GET / HTTP/1.1\nHost: example.com\nConnection: close\r\n\r\n";
    // 获取远程服务器的socket地址(ipv4或ipv6, 内部通过dns解析获取)
    let addrs = "example.com:80".to_socket_addrs().unwrap();
    for addr in addrs {
        info!("socket_addr:{:?}", &addr);
        // 开启本地socket端口
        let socket = TcpSocket::new_v4().unwrap();
        // 连接到远程socket端口
        let mut connect = socket.connect(addr).await.unwrap();
        // 传输数据到远程socket
        connect.write(hyper_text_client_content).await.unwrap();
        let mut buffer = BytesMut::new();
        // 读取远程socket返回的数据
        loop {
            match connect.read_buf(&mut buffer).await {
                Ok(n) => {
                    info!("client receive {} bytes", n);
                    if n==0 { break; }
                }
                Err(e) => {
                    info!("client receive error:{:?}", e)
                }
            }
        }
        let hyper_text_server_content = String::from_utf8(buffer.to_vec()).unwrap();
        info!("example return content bytes:{:?}", &buffer.to_vec());
        info!("example return content:{}", hyper_text_server_content);
    }
}

#[instrument]
async fn tcp_client_time_gov() {
    let addrs = "time.nist.gov:37".to_socket_addrs().unwrap();
    for addr in addrs {
        info!("time socket_addr:{:?}", addr);
        let socket = TcpSocket::new_v4().unwrap();
        let mut connect = socket.connect(addr).await.unwrap();
        let mut buffer = [0;4];
        connect.read_exact(&mut buffer).await.unwrap();
        info!("time nist read bytes:{:?}", u32::from_ne_bytes(buffer));
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
                let mut buffer = BytesMut::new();
                loop {
                    match tcp_stream.read_buf(&mut buffer).await {
                        Ok(n) => {
                            info!("server receive {} bytes", n);
                            let len = &buffer.len();
                            let length = len.add(0);
                            // read_buf ==0 或者数据流以\n\n, \r\n\r\n结尾，则代表流已结束。
                            if n==0 || vec![b'\r', b'\n', b'\r', b'\n'] == &buffer[(length-4)..length] || vec![b'\n', b'\n'] == &buffer[(length-2)..length] {
                                break;
                            }
                        },
                        Err(e) => {
                            info!("server receive error:{:?}", e);
                        }
                    }
                }
                let read_content = String::from_utf8(buffer.to_vec()).unwrap();
                info!("read bytes:{:?}", &buffer.to_vec());
                info!("read content: \n{}", read_content);
                let _ = &tcp_stream.write(b"HTTP/1.1 200 OK\nContent-Type: text/plain\nContent-Length: 12\nConnection: close\n\nHello World!").await.unwrap();
            },
            Err(e) => info!("accept data error, {:?}", e),
        }
    }
}


#[cfg(test)]
mod test {
    use bytes::{Buf, BufMut, BytesMut};

    #[test]
    fn test_bytes_mut() {
        let buf_mut = BytesMut::new();
        println!("buf_mut has remaining :{}", &buf_mut.has_remaining());
        println!("buf_mut has remaining mut:{}", &buf_mut.has_remaining_mut());
        println!("buf_mut remaining mut length: {}", &buf_mut.remaining_mut());
    }
}