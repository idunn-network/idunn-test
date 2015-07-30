#![allow(dead_code)]
#![allow(unused_variables)] 
#![allow(unused_imports)]

extern crate gpgme;
extern crate time;
extern crate rustc_serialize;

mod server;
mod peers;
mod proto;

use std::net::{TcpListener, TcpStream};
use std::thread;
use std::io::prelude::*;

use gpgme::Protocol;
use gpgme::ops;

use server::Server;
use peers::Peers;

fn main() {

    let bind_addr = "127.0.0.1:5432";
    println!("starting server for: {}", bind_addr);
    let s = Server::new(bind_addr);
    let _ = s.thread_handle.join();
}




fn handle_client(stream: TcpStream) {
    let peer_addr = stream.peer_addr();
    println!("Peer addr: {:?}", peer_addr);
    //let mut buf = String::new();
    //let x = stream.read_to_string(&mut buf);

    //let mut buf = [0u8; 128];
    //let x = stream.read(&mut buf);
    //println!("Got {:?} bytes", x.unwrap());
    //println!("Got: {}", std::str::from_utf8(&buf).ok().unwrap());



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


