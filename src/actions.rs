#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_must_use)]
#![allow(unused_mut)]

use peers::Peers;
use client::Client;
use crypto::*;

use rustc_serialize::json::{self, ToJson, Json};

use std::collections::HashMap;

pub fn introduce(keyid: String, address: String) {
    println!("actions::introduce called with {} @ {}", keyid, address);
    let mut client = Client::new(&address[..]);
    let mut hm = HashMap::new();
    hm.insert("message".to_string(), "ping".to_string());
    hm.insert("source".to_string(), "server".to_string());
    let jmsg = json::encode(&hm).unwrap();
    client.send_signed(jmsg);
}

