#![allow(dead_code)]
#![allow(unused_variables)] 
#![allow(unused_imports)]
#![allow(unused_must_use)]
#![allow(unused_mut)]

use peers::Peers;
use client::Client;

pub fn introduce(keyid: String, address: String) {
    println!("actions::introduce called with {} @ {}", keyid, address);
    let mut client = Client::new(&address[..]);
}

