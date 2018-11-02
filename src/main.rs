extern crate dns_parser;
extern crate rand;
use rand::distributions::Alphanumeric;
use rand::prelude::*;
use rand::thread_rng;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use std::net::UdpSocket;
use std::time::Instant;

use dns_parser::{Builder, Packet, ResponseCode};
use dns_parser::{QueryClass, QueryType};

const USIZE: usize = 7;

fn main() {
    let names = [
        "[2001:4860:4860::8888]",
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

#[test]
fn prs2_test() {
    fn tt(sa: Result<SocketAddr, String>) -> String {
        match sa {
            Err(e) => e,
            Ok(x) => x.to_string(),
        }
    }
    assert_eq!(tt(prs2("8.8.8.8")), "8.8.8.8:53");
    assert_eq!(tt(prs2("8.8.8.8:888")), "8.8.8.8:888");
    assert_eq!(
        tt(prs2("[2001:0db8:85a3:0000:0000:8a2e:0370:7334]")),
        "[2001:db8:85a3::8a2e:370:7334]:53"
    );
    assert_eq!(
        tt(prs2("[2001:0db8:85a3:0000:0000:8a2e:0370:7334]:888")),
        "[2001:db8:85a3::8a2e:370:7334]:888"
    );
    assert_eq!(tt(prs2("skydns.ru")), "176.9.59.134:53");
    assert_eq!(
        tt(prs2("kjlkjsdsdlfjsdkfjsldkjfklsdjflskdjfj")),
        "failed to lookup address information: Name or service not known"
    );
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
    let mut arr: [u32; USIZE] = [0; USIZE];
    let sa = prs2(name).expect("An error has occured when parse name");
    let sock = match sa.is_ipv6() {
        true => UdpSocket::bind("[::]:0").expect("Can't bind to local addr"),
        _ => UdpSocket::bind("0.0.0.0:0").expect("Can't bind to local addr"),
    };
    sock.connect(sa).expect("Can't connect to nameserver");
    //sock.local_addr().map(|x| println!("XXX: {}", x.to_string()));

    for i in 0..arr.len() {
        let now = Instant::now();
        let rstr: String = thread_rng().sample_iter(&Alphanumeric).take(8).collect();
        let host = rstr + ".e1.ru";
        //println!("host: {}", host);
        let mut builder = Builder::new_query(1, true);
        builder.add_question(&host, false, QueryType::A, QueryClass::IN);
        let packet = builder.build().map_err(|_| "truncated packet").unwrap();
        sock.send(&packet).expect("Can't send");
        let mut buf = vec![0u8; 4096];
        sock.recv(&mut buf).expect("Recieve from server failed");
        let pkt = Packet::parse(&buf).expect("pkt parse err");

        if pkt.header.response_code == ResponseCode::NoError
            || pkt.header.response_code == ResponseCode::NameError
        {
            arr[i] = now.elapsed().subsec_millis();
        } else {
            panic!("Something bad happening");
        }
    }
    arr.sort();
    let sum: u32 = arr.iter().sum();
    let aver: f32 = sum as f32 / arr.len() as f32;
    let median = arr[arr.len() / 2];
    //let spaces = " ".take(max_len - name.len());
    let spaces = (0..max_len - name.len()).map(|_| " ").collect::<String>();
    println!(
        "name {}{}  average: {}, median: {}",
        name, spaces, aver, median
    );
}
