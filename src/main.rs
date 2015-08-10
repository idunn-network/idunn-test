#![allow(dead_code)]
#![allow(unused_variables)] 
#![allow(unused_imports)]

extern crate gpgme;
extern crate time;
extern crate rustc_serialize;
extern crate toml;

mod server;
mod peers;
mod proto;
mod workqueue;
mod actions;
mod client;
mod crypto;
mod config;

use std::net::{TcpListener, TcpStream};
use std::thread;
use std::io::prelude::*;
use std::sync::{Arc, Mutex};

use gpgme::Protocol;
use gpgme::ops;

use server::Server;
use peers::Peers;
use workqueue::WorkQueue;
use workqueue::worker_thread;
use config::read_config;
use toml::*;

fn main() {
    let conf = read_config();
    start_srv(conf);
}

fn keylist() {
    let proto = gpgme::PROTOCOL_OPENPGP;
    let mut mode = ops::KeyListMode::empty();
    mode.insert(ops::KEY_LIST_MODE_LOCAL);

    let mut ctx = gpgme::create_context().unwrap();
    ctx.set_protocol(proto).unwrap();
    ctx.set_key_list_mode(mode).unwrap();

    let searchkeys = vec![""];
    let mut keys = ctx.find_keys(searchkeys).unwrap();

    for key in keys.by_ref().filter_map(Result::ok) {
        println!("keyid    : {}", key.id().unwrap_or("?"));
        println!("fpr      : {}", key.fingerprint().unwrap_or("?"));
        for (i, user) in key.user_ids().enumerate() {
            println!("userid   : {} {}", i, user.uid().unwrap_or("[none]"));
            println!("valid    : {} {:?}", i, user.validity())
        }
        println!("");
    }

}

#[test]
fn peer_basics () {
    let mut p = Peers::new();
    p.add("testaddr".to_string(),"testkey".to_string(),false);
    println!("peerlist: {:?}", p);
    p.add("testaddr2".to_string(),"testkey2".to_string(),true);
    println!("peerlist: {:?}", p);
}

//#[test]
//fn testkeylist(){
//    keylist();
//}


fn start_srv(conf: Option<toml::Value> ) {
    let mut bind_addr: String;
    //TODO: must be some better way to handle this.
    match conf {
        Some(t) => {
            bind_addr = t.lookup("network.bind_address").unwrap().as_str().unwrap().to_string();
        }
        None => {
            bind_addr = "127.0.0.1:5432".to_string();
        }
    }

    let mut wq = Arc::new(Mutex::new(WorkQueue::new()));

    let worker_thread = {
        let wq = wq.clone();
        thread::spawn(|| {worker_thread(wq)});
    };

    println!("starting server for: {}", bind_addr);
    let s = Server::new(&bind_addr, wq);
    

    let _ = s.thread_handle.join();
    //let _ = worker_thread.thread_handle.join();
}


