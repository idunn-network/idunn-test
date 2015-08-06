#![allow(dead_code)]
#![allow(unused_variables)] 
#![allow(unused_imports)]

use std::collections::{VecDeque, VecMap};
use std::io::{self, BufRead,Write, BufReader, Read};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::thread::{self, JoinHandle};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Receiver, TryRecvError};
use time::Tm;

use peers::Peers;
use workqueue::WorkQueue;
use proto::handle_client;

pub struct Client {
    address: String,
    stream: Option<TcpStream>,
}

impl Client {
    pub fn new(addr: &str) -> Client {
        let stream = TcpStream::connect(addr);
        let addr = addr.to_string();
        match stream {
            Ok(s) => { Client{ address: addr, stream: Some(s) } }
            Err(e) => {
                println!("failed to connect to {} because: {:?}", addr, e);
               Client{ address: addr, stream: None}
            }
        }
    }
    pub fn send(&mut self, message: String) -> String{
        match self.stream {
            Some(ref mut stream) =>  {
                let _ = stream.write(&message.into_bytes()[..]);
                let mut buf = String::new();
                let _ = stream.read_to_string(&mut buf);
                buf
            }
            None => { println!("attempted to send data, but Client {} has no TcpStream", self.address); "error".to_string() }
        }

    }
}

