#![allow(dead_code)]
#![allow(unused_variables)] 
#![allow(unused_imports)]

extern crate time;
extern crate gpgme;

use std::net::{TcpStream, ToSocketAddrs};
use peers::Peers;
use std::error;
use std::sync::{Arc, Mutex};
use std::io::BufReader;
use std::io;
use std::io::prelude::*;
use gpgme::Protocol;
use gpgme::Data;
use gpgme::ops;

use std::collections::HashMap;

use rustc_serialize::json::{self, ToJson, Json};

use time::Timespec;
use time::now;

/*
 * idunn: 1.0
 * nodekeyid: .....
 * sig-length: xxx
 * data-length: xxx
 * begin:
 * [sig][data]
 */

/*
 * messages:
 * ping [source: server|client] (so we know whether to make note of it as a peer
 * where-is [keyid] [ttl]
 * encrypted: list-services
 * peer-list [list of peer keys and addresses]
 * introduce [keyid] [address] (this one is from a cilent)
 */

//enum ProtocolError { InvalidMagic, UnsupportedVersion }

pub fn handle_client(stream: TcpStream, mut peers: Arc<Mutex<Peers>>) -> Result<String,String>{
    //read first line
    //switch based on version

    let addrstr = stream.peer_addr().unwrap().to_string();
    let mut items = addrstr.split(":");
    peers.lock().unwrap().add(addrstr.clone(), "testkey".to_string(), true);
    println!("{:?}",peers);
    println!("addr: {:?}...{:?}",items.next(), items.next());

    let mut buf = String::new();
    let mut br = BufReader::new(stream);
    let bytes = match br.read_line(&mut buf) {
        Ok(size) =>  println!("read: {}", buf),
        Err(err) =>  return Err(err.to_string())
    };
    let mut items = buf.trim().split(": ");
    if items.clone().count().ne(&2) { return Err("Invalid header; expected: idunn: <version>".to_string()); }
    let a = items.next().unwrap();
    let b = items.next().unwrap();
    println!("header: {:?}...{:?}",a, b);
    if a.ne(&"idunn".to_string()) {println!("invalid header"); 
        return Err("Invalid header; expected: idunn: <version>".to_string());} 
//    if b.ne(&"1.0:".to_string()) {println!("invalid version"); return Err("Unsupported version".to_string())};
    if b.eq(&"1.0".to_string()) {return handle_1_0(br, peers)}
   
    println!("bad version");
    Err("impossible".to_string())
}

pub fn read_headers (br: &mut BufReader<TcpStream>) -> Option<HashMap<String, String>> {
    let mut m: HashMap<String, String> = HashMap::new();
    loop {
        let mut buf = String::new();
        br.read_line(&mut buf);
        if buf.len().eq(&1usize) { break }
        let v: Vec<&str> = buf.trim().splitn(2,": ").collect();
        if v.len().eq(&2usize) { m.insert(v[0].to_string(), v[1].to_string()); }
    }
    Some(m)
}

pub fn from_json(json_str: &str) -> HashMap<String, String> {
    match json::decode(json_str) {
        Ok(s) => { return s; }
        Err(e) => { println!("json decode failed: {:?}", e);
            return HashMap::new();
        }
    }
}

pub fn read_body (br: &mut BufReader<TcpStream>, headers: & HashMap<String, String>) -> Result<Option<HashMap<String,String>>, String> {
    let mut sl = 0usize;
    let mut dl = 0usize;
    let mut el = 0usize;

    if headers.contains_key("sig-length") {
        let tmp = headers.get("sig-length").unwrap();
        sl = tmp.parse::<usize>().unwrap();
    }
    if headers.contains_key("data-length") {
        let tmp = headers.get("data-length").unwrap();
        dl = tmp.parse::<usize>().unwrap();
    }
    if headers.contains_key("encrypted-length") {
        let tmp = headers.get("encrypted-length").unwrap();
        el = tmp.parse::<usize>().unwrap();
    }

    println!("sl, dl, el: {}, {}, {}", sl, dl, el);

    if sl > 0 && dl > 0{ 
        println!("running verification"); 
        let mut sbuf = vec! [0u8; sl];
        br.read(&mut sbuf);
        //let sig = String::from_utf8(sbuf).unwrap();
        let mut dbuf = vec! [0u8; dl];
        br.read(&mut dbuf);
        //let dat = String::from_utf8(sbuf).unwrap();
        println!("sig: {:?}",sbuf.clone());
        println!("dat: {:?}",String::from_utf8(dbuf.clone()).unwrap());
        let vresult = verify(&dbuf,sbuf);
        println!("verified: {:?}", vresult);
        let h = from_json(&String::from_utf8(dbuf).unwrap());
        println!("hm: {:?}", h);
        return Ok(Some(h));

    } else if el > 0  {
        println!("decrypting"); 
        let mut ebuf = vec! [0u8; el];
        br.read(&mut ebuf);
        println!("edata: {:?}",ebuf.clone());
        match decrypt(ebuf) {
            Ok(data) => {
                let s = &data.into_string().unwrap().unwrap();
                println!("decrypted dat: {}", s);
                let h = from_json(s);
                println!("hm: {:?}", h);
                return Ok(Some(h));
            }
            Err(err) => {
                println!("decrypt error: {:?}", err);
                return Err("failed".to_string());
            }
        }
    }

    Err("impossible".to_string())
}
pub fn decrypt(dat: Vec<u8>) -> Result<gpgme::Data<'static>, gpgme::error::Error>{
    let proto = Protocol::OpenPgp;
    let mut ctx = gpgme::create_context().unwrap();
    ctx.set_protocol(proto).unwrap();
    let mut edat = Data::from_bytes(dat).unwrap();
    let mut outdat = Data::new().unwrap();
    match ctx.decrypt(&mut edat, &mut outdat) {
        Ok(..) => { return Ok(outdat); }
        Err(err) => { return Err(err); }
    }
}


pub fn verify(dat: &Vec<u8>, sig: Vec<u8>) -> Result<gpgme::ops::VerifyResult, gpgme::error::Error>{
    let proto = Protocol::OpenPgp;
    let mut mode = ops::KeyListMode::empty();
    mode.insert(ops::KEY_LIST_MODE_LOCAL);

    let mut ctx = gpgme::create_context().unwrap();
    ctx.set_protocol(proto).unwrap();
    let mut sigdat = Data::from_bytes(sig).unwrap();
    let mut datdat = Data::from_bytes(dat).unwrap();
    let v1 = ctx.verify(&mut sigdat, None, Some(&mut datdat) );
    //this wasn't the correct call order - it's used for inline sigs, I think:
    //let v2 = ctx.verify(&mut sigdat, Some(&mut datdat) , None);
    //println!("verify results: {:?} {:?}", v1, v2);

    for (i, sig) in v1.clone().unwrap().signatures().enumerate() {
        println!("signature {}", i);
        println!("     stat {}", sig.status());
        println!("    valid {:?}", sig.validity());
        println!("   reason {}", sig.validity_reason());
    }

    v1

}


pub fn handle_1_0 (mut br: BufReader<TcpStream>, mut peers: Arc<Mutex<Peers>>) -> Result<String,String> {
    println!("1.0....");    

    let mut headers = read_headers(&mut br).unwrap();
    println!("I got headers: {:?}", headers);
    let mut b = read_body(&mut br, &headers);

    println!("handle_1_0: {:?}", b);
    
    Err("impossible 1.0".to_string())
}



