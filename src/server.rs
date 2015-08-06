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
use workqueue::WorkQueue;
use proto::handle_client;

pub struct Server {
    listener: TcpListener,
    pub thread_handle: JoinHandle<()>,
    peers: Arc<Mutex<Peers>>,
    workqueue: Arc<Mutex<WorkQueue>>
}

impl Server {
    pub fn new(addr: &str, wq: Arc<Mutex<WorkQueue>>) -> Server {
        // .bind() -> Result<TcpListener>
        // .accept() -> Result<(TcpStream, SocketAddr)>
        let listener = TcpListener::bind(addr).unwrap();
        println!("binding {}", addr);
       
        let mut p = Arc::new(Mutex::new(Peers::new()));
        let server_thread = {
            let p = p.clone();
            let wq = wq.clone();
            let listener = listener.try_clone().unwrap();
            thread::spawn(|| { Server::server_thread(listener, p, wq);})
        };


        Server{
            listener: listener,
            thread_handle: server_thread,
            peers: p,
            workqueue: wq
        }
    }
    fn server_thread(listener: TcpListener, peers: Arc<Mutex<Peers>>,
                     wq: Arc<Mutex<WorkQueue>>) {
        println!("starting thread");
        loop {
            let (stream, addr) = listener.accept().unwrap();

            println!("client socketaddr: {:?}", addr);
            let handler_thread = {
                let p = peers.clone();
                let wq = wq.clone();
                thread::spawn(|| { handle_client(stream, p, wq);})
            };

        }
    }
    fn print_peers(&self) {
        println!("{:?}", self.peers);
    }
}

