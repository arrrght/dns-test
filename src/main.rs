extern crate dns_parser;
use std::net::IpAddr;
use std::net::UdpSocket;
use std::time::Instant;

use dns_parser::rdata::a::Record;
use dns_parser::{Builder, Packet, RData, ResponseCode};
use dns_parser::{QueryClass, QueryType};

fn main() {
    let mut arr: [u32; 10] = [0; 10];
    let sock = UdpSocket::bind("127.0.0.1:0").unwrap();
    sock.connect("127.0.0.1:53").unwrap();

    for i in 0..9 {
        let now = Instant::now();
        let host = "e1.ru";
        let mut builder = Builder::new_query(1, true);
        builder.add_question(host, false, QueryType::A, QueryClass::IN);
        let packet = builder.build().map_err(|_| "truncated packet").unwrap();
        sock.send(&packet);
        let mut buf = vec![0u8; 4096];
        sock.recv(&mut buf).unwrap();
        let pkt = Packet::parse(&buf).unwrap();

        if pkt.header.response_code != ResponseCode::NoError {
            assert!(true, false); //TODO
        }
        if pkt.answers.len() == 0 {
            assert!(true, false); //TODO
        }
        for ans in pkt.answers {
            match ans.data {
                RData::A(Record(ip)) => {
                    arr[i] = now.elapsed().subsec_micros();
                    println!("found: {}, time:{}", ip, arr[i]);
                }
                _ => {}
            }
        }

    }
    arr.sort();
    let sum: u32 = arr.iter().sum();
    let aver: f32 = sum as f32 / arr.len() as f32;
    let median = arr[arr.len() / 2];
    println!("average: {}, median: {}", aver, median);
}
