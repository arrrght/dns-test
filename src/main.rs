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
    // TODO: ipv6
    let names = ["8.8.8.8", "1.1.1.1:53"];
    for i in &names {
        doit(i);
    }
}
fn doit(name: &str) {
    let mut arr: [u32; USIZE] = [0; USIZE];
    let sa: SocketAddr = match name.to_socket_addrs() {
        Err(_) => format!("{}:53", name).parse().unwrap(),
        Ok(_) => name.parse().unwrap(),
    };

    //println!("port: {}", sa.port());
    let sock = UdpSocket::bind("0.0.0.0:0").expect("Can't bind to local addr");
    sock.connect(sa).expect("Can't connect to nameserver");

    for i in 0..arr.len() {
        let now = Instant::now();
        let rstr: String = thread_rng().sample_iter(&Alphanumeric).take(8).collect();
        let host = rstr + ".e1.ru";
        println!("host: {}", host);
        let mut builder = Builder::new_query(1, true);
        builder.add_question(&host, false, QueryType::A, QueryClass::IN);
        let packet = builder.build().map_err(|_| "truncated packet").unwrap();
        sock.send(&packet).expect("Can't send");
        let mut buf = vec![0u8; 4096];
        sock.recv(&mut buf).expect("Recieve from server failed");
        let pkt = Packet::parse(&buf).unwrap();

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
    println!("name {}: average: {}, median: {}", name, aver, median);
}
