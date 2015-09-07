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
use actions::*;

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

pub fn dispatch(it: IdunnTask) {
    match it {
        IdunnTask::Ping{address: address} =>
            { println!("I should ping {}", address); }
        IdunnTask::Introduce{keyid: keyid, address: address} =>
            {
                println!("I will introduce {} at {}", keyid, address);
                introduce(keyid, address);
            }
        _ => {}
    }
}

pub fn worker_thread(wq: Arc<Mutex<WorkQueue>>) {
    loop {
        println!("in the q: {:?}", wq.lock().unwrap().queue);
        println!("count: {}", wq.lock().unwrap().queue.len());
        let result = wq.lock().unwrap().queue.pop();
        match result{
            Some(item) => {
                dispatch(item);
                sleep_ms(10u32)
            }
            None => { sleep_ms(3000u32); }
        }
    }
}


