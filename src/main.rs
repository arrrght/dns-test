extern crate dns_parser;
extern crate futures;
extern crate rand;
extern crate tokio;

use dns_parser::{Builder, Packet, ResponseCode};
use dns_parser::{QueryClass, QueryType};
use rand::distributions::Alphanumeric;
use rand::prelude::*;
use rand::thread_rng;
use std::net::{SocketAddr, ToSocketAddrs};
use std::time::Instant;

//use futures::prelude::*;
//use futures::future;
use std::rc::Rc;
use tokio::net::UdpSocket;
use tokio::prelude::*;

const USIZE: usize = 1;

fn main() {
    let names = [
        //"[2001:4860:4860::8888]",
        "8.8.8.8",
        "193.58.251.251",
        //"1.1.1.1",
        "9.9.9.9",
    ];
    let max_len: usize = names
        .iter()
        .fold(0, |mx, x| if x.len() > mx { x.len() } else { mx });
    for name in &names {
        doit(max_len, name);
    }
}

fn prs2(name: &str, port: usize) -> Result<SocketAddr, String> {
    match name.to_socket_addrs() {
        Err(_) => match format!("{}:{}", name, port).to_socket_addrs() {
            Err(e) => Err(e.to_string()),
            Ok(o) => o.clone().next().ok_or_else(|| "null?".to_owned()),
        },
        Ok(o) => o.clone().next().ok_or_else(|| "null?".to_owned()),
    }
}

fn build_pkt(host: &str) -> Result<Vec<u8>, Vec<u8>> {
    let mut builder = Builder::new_query(1, true);
    builder.add_question(&host, false, QueryType::A, QueryClass::IN);
    builder.build()
}

fn doit(max_len: usize, name: &str) {
    let mut arr: [u32; USIZE] = [0; USIZE];
    let r = Rc::new(arr);
    let remote_sock = prs2(name, 53).expect("An error has occured when parse name");
    let local_sock = match remote_sock.is_ipv6() {
        true => prs2("[::]:0", 0),
        _ => prs2("0.0.0.0:0", 0),
    }.expect("Err in create local_sock");

    for i in 0..arr.len() {
        let local_bind = UdpSocket::bind(&local_sock).expect("Can't bind context");
        let now = Instant::now();
        let rstr: String = thread_rng().sample_iter(&Alphanumeric).take(8).collect();
        let host = rstr + ".e1.ru";
        let packet = build_pkt(&host).expect("Can't build packet");
        let mut buf = vec![0u8; 4096];
        let send_future = local_bind.send_dgram(packet, &remote_sock);

        let task = send_future
            .and_then(|(socket, _buf)| {
                println!("Send ok, ttl: {:?}", socket.ttl());
                socket.recv_dgram(buf)
            }).map_err(|e| println!("send failed, err: {}", e));

        let task2 = task.and_then(|(_sock_local, data, _len, _sock_remote)| {
            let pkt = Packet::parse(&data).expect("pkt parse err");
            if pkt.header.response_code == ResponseCode::NoError
                || pkt.header.response_code == ResponseCode::NameError
            {
                //elapsed = now.elapsed().subsec_millis();
            } else {
                panic!("Something bad happening");
            }
            //futures::future::ok::<u32, u32>(42)
            Ok(())
            //futures::future::ok::<_, ()>(String::from("hello"))
            //Ok(futures::future::ok(2);
        });

        fn add_10<F>(f: F) -> impl Future<Item = i32, Error = F::Error>
        where
            F: Future<Item = i32>,
        {
            f.map(|i| i + 10)
        }

        let future = task2
            .timeout(std::time::Duration::from_millis(100))
            .map_err(|e| println!("error = {:?}", e));

        //let f2 = future::ok::<u32, ()>(1);
        //let f = future::empty::<(), ()>().select2(f2);

        // let some: u32 = 42;
        // let f = futures::future::ok(some).map(|x| {
        //     println!("{}", x);
        //     future.wait();
        // });

        tokio::run(future);

        // let process = local_bind
        //     .send_dgram(packet, &remote_sock)
        //     .and_then(|(socket, _)| socket.recv_dgram(buf))
        //     .map(|(_, data, _len, _)| {
        //         let pkt = Packet::parse(&data).expect("pkt parse err");

        //         if pkt.header.response_code == ResponseCode::NoError
        //             || pkt.header.response_code == ResponseCode::NameError
        //         {
        //             arr[i] = now.elapsed().subsec_millis();
        //         } else {
        //             panic!("Something bad happening");
        //         }
        //     }).wait();
        // match process {
        //     Ok(_) => {}
        //     Err(e) => println!("Process err: {}", e),
        // };
    }
    arr.sort();
    let sum: u32 = arr.iter().sum();
    let aver: f32 = sum as f32 / arr.len() as f32;
    let median = arr[arr.len() / 2];
    let spaces = (0..max_len - name.len()).map(|_| " ").collect::<String>();
    println!("{}{}  average: {}, median: {}", name, spaces, aver, median);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};
    #[test]
    fn build_pkt_test() {
        // TODO
        assert!(true);
        //let pkt = build_pkt("abc.com");
    }
    #[test]
    fn prs2_test() {
        assert_eq!(prs2("8.8.8.8", 53).is_ok(), true);
        assert_eq!(prs2("8.8.8.8", 53), prs2("8.8.8.8:53", 999));
        assert_eq!(
            prs2("8.8.8.8", 53).ok(),
            Some(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)), 53))
        );
        assert_eq!(
            prs2("[2001:0db8:85a3:0000:0000:8a2e:0370:7334]", 53).is_ok(),
            true
        );
        match prs2("[2001:0db8:85a3:0000:0000:8a2e:0370:7334]", 53) {
            Err(_) => assert!(false, "convert host failed"),
            Ok(x) => assert_eq!(x.port(), 53),
        };

        assert_eq!(
            prs2("[2001:0db8:85a3:0000:0000:8a2e:0370:7334]", 53),
            prs2("[2001:db8:85a3::8a2e:370:7334]:53", 53)
        );
        assert_eq!(
            prs2("[2001:0db8:85a3:0000:0000:8a2e:0370:7334]:888", 53),
            prs2("[2001:db8:85a3::8a2e:370:7334]:888", 53)
        );
        assert_eq!(prs2("skydns.ru", 53).is_ok(), true);
        match prs2("skydns.ru", 53) {
            Err(_) => assert!(false, "convert host failed"),
            Ok(x) => assert_eq!(x.port(), 53),
        };
        assert_eq!(
            prs2("kjlkjsdsdlfjsdkfjsldkjfklsdjflskdjfj", 53).is_err(),
            true
        );
    }
}
