#![allow(dead_code)]
#![allow(unused_variables)] 
#![allow(unused_imports)]

extern crate time;
use rustc_serialize::json::{self, ToJson, Json};

use time::Timespec;
use time::now;
use std::collections::BTreeMap;
use std::io::{Read,Write};
use std::fs::File;
use std::str::from_utf8;

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash, RustcDecodable, RustcEncodable)]
pub struct Peer {
    address: String,
    key_id: String,
    last_seen: time::Timespec,
    offline: bool
}

#[derive(Debug, PartialEq, Eq, RustcDecodable, RustcEncodable)]
pub struct Peers {
    peers: BTreeMap<String,Peer>,
} 

impl Peer {
    pub fn new_with_time(addr: String, key: String, last: time::Timespec, offline: bool) -> Peer  {
        Peer {
            address: addr,
            key_id: key,
            last_seen: last,
            offline: offline
        }
    }
    pub fn new(addr: String, key: String, offline: bool) -> Peer  {
        Peer {
            address: addr,
            key_id: key,
            last_seen: time::get_time(),
            offline: offline
        }
    }
}

impl Peers {
    pub fn new() -> Peers {
        Peers {
            peers: BTreeMap::new()
        }
    }
    pub fn add(&mut self, addr: String, key: String, offline: bool) {
        let p = Peer::new(addr.clone(), key, offline);
        self.peers.insert(addr, p);
    }
    pub fn json_str(&self) -> String {
        let encoded = json::encode(&self).unwrap();
        encoded
    }
    pub fn new_from_json(json_str: &str) -> Peers{
        json::decode(json_str).unwrap()
    }
    pub fn to_file(&self, file: String) {
        let mut f: File = File::create(file).unwrap();
        f.write(self.json_str().as_bytes());
    }
    pub fn new_from_file(file: String) -> Peers {
        let mut f: File = File::open(file).unwrap();
        let mut s: String = String::new();
        let _ = f.read_to_string(&mut s);
        Peers::new_from_json(&s)
    }
}

#[test]
fn peer_serialize () {
    let mut p = Peers::new();
    p.add("testaddr".to_string(),"testkey".to_string(),false);
    let jstr = p.json_str();
    let pnew = Peers::new_from_json(&jstr);
    assert!(p.eq(&pnew));
    p.to_file("peers.txt".to_string());
    let pfile = Peers::new_from_file("peers.txt".to_string());
    assert!(p.eq(&pfile));
}

