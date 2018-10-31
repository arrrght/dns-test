extern crate dns_lookup;
use dns_lookup::lookup_host;
use std::net::IpAddr;
use std::time::Instant;

fn main() {
    let mut arr: [u32; 10] = [0; 10];
    for i in 0..9 {
        let now = Instant::now();
        let host = "google.com";
        let ips: Vec<IpAddr> = lookup_host(&host).unwrap();
        arr[i] = now.elapsed().subsec_micros();
        println!("{}, time:{}", ips[0], arr[i]);
    }
    arr.sort();
    let sum: u32 = arr.iter().sum();
    let aver: f32 = sum as f32 / arr.len() as f32;
    let median = arr[arr.len() / 2];
    
    println!("average: {}, median: {}", aver, median);
}
