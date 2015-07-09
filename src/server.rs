#![allow(dead_code)]
#![allow(unused_variables)] 
#![allow(unused_imports)]

use std::collections::{VecDeque, VecMap};
use std::io::{self, BufRead,Write, BufReader};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::thread::{self, JoinHandle};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Receiver, TryRecvError};
use time::Tm;
use peers::Peers;

use proto::handle_client;

pub struct Server {
    listener: TcpListener,
    pub thread_handle: JoinHandle<()>,
    peers: Arc<Mutex<Peers>>,
}

impl Server {
    pub fn new(addr: &str) -> Server {
        // .bind() -> Result<TcpListener>
        // .accept() -> Result<(TcpStream, SocketAddr)>
        let listener = TcpListener::bind(addr).unwrap();
        println!("binding {}", addr);
       
        let mut p = Arc::new(Mutex::new(Peers::new()));
        let server_thread = {
            let p = p.clone();
            let listener = listener.try_clone().unwrap();
            thread::spawn(|| { Server::server_thread(listener, p);})
        };


        Server{
            listener: listener,
            thread_handle: server_thread,
            peers: p,
        }
    }
    fn server_thread(listener: TcpListener, peers: Arc<Mutex<Peers>>) {
        println!("starting thread");
        loop {
            let (stream, addr) = listener.accept().unwrap();

            println!("client socketaddr: {:?}", addr);
            let handler_thread = {
                let p = peers.clone();
                thread::spawn(|| { handle_client(stream, p);})
            };

        }
    }
    fn print_peers(&self) {
        println!("{:?}", self.peers);
    }
}

#[test]
fn start_srv() {
    let bind_addr = "127.0.0.1:5432";
    println!("starting server for: {}", bind_addr);
    let s = Server::new(bind_addr);
    let _ = s.thread_handle.join();
}


