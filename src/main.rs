extern crate dns_parser;
extern crate rand;
extern crate futures;

use std::thread;
use rand::distributions::Alphanumeric;
use rand::prelude::*;
use rand::thread_rng;
use std::time::Instant;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use dns_parser::{Builder, Packet, ResponseCode};
use dns_parser::{QueryClass, QueryType};
use futures::Future;
//use futures::sync::oneshot;
use futures::future::join_all;

const USIZE: usize = 15;

fn main() {
    let names = [
        //"[2001:4860:4860::8888]",
        "8.8.8.8",
        "193.58.251.251",
        "1.1.1.1",
        "9.9.9.9",
    ];
    let max_len: usize = names
        .iter()
        .fold(0, |mx, x| if x.len() > mx { x.len() } else { mx });
    for name in &names {
        doit(max_len, name);
    }
}


fn prs2(name: &str) -> Result<SocketAddr, String> {
    match name.to_socket_addrs() {
        Err(_) => match format!("{}:53", name).to_socket_addrs() {
            Err(e) => Err(e.to_string()),
            Ok(o) => o.clone().next().ok_or_else(|| "null?".to_owned()),
        },
        Ok(o) => o.clone().next().ok_or_else(|| "null?".to_owned()),
    }
}

fn doit(max_len: usize, name: &str) {
    //let mut arr: [u32; USIZE] = [0; USIZE];
    let sa = prs2(name).expect("An error has occured when parse name");

    let mut rx_set = Vec::new();
    for _i in 0..USIZE {
        let (tx, rx) = futures::oneshot();
        rx_set.push(rx);
        thread::spawn(move || {
            let sock = match sa.is_ipv6() {
                true => UdpSocket::bind("[::]:0"),
                _ => UdpSocket::bind("0.0.0.0:0"),
            }.expect("Can't bind context");
            sock.connect(sa).expect("Can't connect to nameserver");
            let now = Instant::now();
            let rstr: String = thread_rng().sample_iter(&Alphanumeric).take(8).collect();
            let host = rstr + ".e1.ru";
            let mut builder = Builder::new_query(1, true);
            builder.add_question(&host, false, QueryType::A, QueryClass::IN);
            let packet = builder.build().expect("Can't build packet");
            sock.send(&packet).expect("Can't send");
            let mut buf = vec![0u8; 4096];
            sock.recv(&mut buf).expect("Recieve from server failed");
            let pkt = Packet::parse(&buf).expect("pkt parse err");

            if pkt.header.response_code == ResponseCode::NoError
                || pkt.header.response_code == ResponseCode::NameError
            {
                let _ = tx.send(now.elapsed().subsec_millis());
            } else {
                panic!("Something bad happening");
            }
        });
    }
    let mut arr = join_all(rx_set).wait().unwrap();
    arr.sort();
    println!("result: {:?}", arr);
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
    fn prs2_test() {
        assert_eq!(prs2("8.8.8.8").is_ok(), true);
        assert_eq!(prs2("8.8.8.8"), prs2("8.8.8.8:53"));
        assert_eq!(
            prs2("8.8.8.8").ok(),
            Some(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)), 53))
        );
        assert_eq!(
            prs2("[2001:0db8:85a3:0000:0000:8a2e:0370:7334]").is_ok(),
            true
        );
        match prs2("[2001:0db8:85a3:0000:0000:8a2e:0370:7334]") {
            Err(_) => assert!(false, "convert host failed"),
            Ok(x) => assert_eq!(x.port(), 53),
        };

        assert_eq!(
            prs2("[2001:0db8:85a3:0000:0000:8a2e:0370:7334]"),
            prs2("[2001:db8:85a3::8a2e:370:7334]:53")
        );
        assert_eq!(
            prs2("[2001:0db8:85a3:0000:0000:8a2e:0370:7334]:888"),
            prs2("[2001:db8:85a3::8a2e:370:7334]:888")
        );
        assert_eq!(prs2("skydns.ru").is_ok(), true);
        match prs2("skydns.ru") {
            Err(_) => assert!(false, "convert host failed"),
            Ok(x) => assert_eq!(x.port(), 53),
        };
        assert_eq!(prs2("kjlkjsdsdlfjsdkfjsldkjfklsdjflskdjfj").is_err(), true);
    }
}

