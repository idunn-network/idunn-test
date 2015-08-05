#![allow(dead_code)]
#![allow(unused_variables)] 
#![allow(unused_imports)]

use std::io::{self, BufRead,Write, BufReader};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Receiver, TryRecvError};
use std::thread::{self, JoinHandle, sleep_ms};
use time::Tm;
use peers::Peers;
use std::collections::HashMap;

#[derive(Debug)]
pub enum IdunnTask { 
    Ping{address: String},
    Introduce{keyid: String, address: String},
    WhereIs{keyid: String},
    PeerList 
}


#[derive(Debug)]
pub struct WorkQueue {
    pub queue: Vec<IdunnTask>,
}

impl WorkQueue {
    pub fn new() -> WorkQueue {
        WorkQueue{
            queue: Vec::new(),
        }
    }
    pub fn insert(&mut self, it: IdunnTask) {
        println!("inserting {:?}", it);
        self.queue.push(it);
    }
}

pub fn worker_thread(wq: Arc<Mutex<WorkQueue>>) {
    loop {
        println!("in the q: {:?}", wq.lock().unwrap().queue);
        sleep_ms(1000u32);
    }
}


