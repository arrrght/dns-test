use futures::{Async, Future, Poll};
use dns_parser::Builder;
use dns_parser::{QueryClass, QueryType};
use rand::distributions::Alphanumeric;
use rand::prelude::*;
use rand::thread_rng;

pub struct PB;
impl Future for PB {
    type Item = std::vec::Vec<u8>;
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let rstr: String = thread_rng().sample_iter(&Alphanumeric).take(8).collect();
        let host = rstr + ".e1.ru";
        let mut builder = Builder::new_query(1, true);
        builder.add_question(&host, false, QueryType::A, QueryClass::IN);
        let packet = builder.build().expect("Can't build packet");
        Ok(Async::Ready(packet))
    }
}


//pub struct HelloWorld;
//impl Future for HelloWorld {
//    type Item = String;
//    type Error = ();
//
//    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
//        Ok(Async::Ready("helloworld".to_string()))
//    }
//}
