#![allow(dead_code)]
#![allow(unused_variables)] 
#![allow(unused_imports)]
#![allow(unused_must_use)]
#![allow(unused_mut)]

extern crate time;
extern crate gpgme;

use peers::Peers;
use workqueue::*;
//use crypto::*;

use std::net::{TcpStream, ToSocketAddrs};
use std::error;
use std::sync::{Arc, Mutex};
use std::io::BufReader;
use std::io::BufWriter;
use std::io;
use std::io::prelude::*;
use gpgme::Protocol;
use gpgme::Data;
use gpgme::ops;
use gpgme::keys;
use gpgme::context::*;

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

pub fn handle_client(stream: TcpStream, mut peers: Arc<Mutex<Peers>>,
                     mut wq: Arc<Mutex<WorkQueue>>) -> Result<String,String>{
    //read first line
    //switch based on version

    let addrstr = stream.peer_addr().unwrap().to_string();
    let mut items = addrstr.split(":");
    peers.lock().unwrap().add(addrstr.clone(), "testkey".to_string(), true);
    println!("{:?}",peers);
    println!("addr: {:?}...{:?}",items.next(), items.next());

    let mut buf = String::new();
    let mut br = BufReader::new(stream.try_clone().unwrap());
    let mut bw = BufWriter::new(stream.try_clone().unwrap());
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
    if b.eq(&"1.0".to_string()) {return handle_1_0(br, bw, peers, wq)}
   
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
            Ok(d) => {
                let data = d.data;
                let dr = d.dr;
                let key = dr.recipients().next().unwrap().key_id().unwrap();
                println!("sender keyid: {}", key);
                let s = &data.into_string().unwrap();
                println!("decrypted dat: {}", s);
                let mut h = from_json(s);
                h.insert("keyid".to_string(),key.to_string());
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

pub struct Decrypted {
    data: gpgme::Data<'static>,
    dr: gpgme::ops::DecryptResult
}

pub fn decrypt(dat: Vec<u8>) -> Result<Decrypted, gpgme::error::Error>{
    let proto = gpgme::PROTOCOL_OPENPGP;
    let mut ctx = gpgme::create_context().unwrap();
    ctx.set_protocol(proto).unwrap();
    let mut edat = Data::from_bytes(&dat).unwrap();
    let mut outdat = Data::new().unwrap();
    match ctx.decrypt(&mut edat, &mut outdat) {
        Ok(dr) => { return Ok( Decrypted {data: outdat, dr: dr}); }
        Err(err) => { return Err(err); }
    }
}


pub fn verify(dat: &Vec<u8>, sig: Vec<u8>) -> Result<gpgme::ops::VerifyResult, gpgme::error::Error>{
    let proto = gpgme::PROTOCOL_OPENPGP;
    let mut mode = ops::KeyListMode::empty();
    mode.insert(ops::KEY_LIST_MODE_LOCAL);

    let mut ctx = gpgme::create_context().unwrap();
    ctx.set_protocol(proto).unwrap();
    let mut sigdat = Data::from_bytes(&sig).unwrap();
    let mut datdat = Data::from_bytes(dat).unwrap();
    let v1 = ctx.verify(&mut sigdat, None, Some(&mut datdat) );
    //this wasn't the correct call order - it's used for inline sigs, I think:
    //let v2 = ctx.verify(&mut sigdat, Some(&mut datdat) , None);
    //println!("verify results: {:?} {:?}", v1, v2);

    for (i, sig) in v1.clone().unwrap().signatures().enumerate() {
        println!("signature {}", i);
        println!("     stat {:?}", sig.status());
        println!("    valid {:?}", sig.validity());
        println!("   reason {:?}", sig.validity_reason());
    }

    v1
}
/*
pub fn passphrase_cb(_hint: Option<&str>, _info: Option<&str>,
                     _prev_was_bad: bool, out: &mut Write) -> gpgme::Result<()> {
    try!(out.write_all(b"abc\n"));
    Ok(())
}
*/

pub fn sign(dat: &String) -> Data {
    let proto = gpgme::PROTOCOL_OPENPGP;
    let mut ctx = gpgme::create_context().unwrap();
    ctx.set_protocol(proto);
    //let mut guard = ctx.with_passphrase_cb();

    let mut input = Data::from_bytes(dat).unwrap();
    let mut output = Data::new().unwrap();

    let result = ctx.sign_detached(&mut input, &mut output);
    //result.signatures().next().unwrap();

    println!("sign result: {:?}", result.clone().unwrap().signatures().next().unwrap());

    output
}

pub fn encrypt(keystr: String, dat: &String) -> Data<'static> {
    let proto = gpgme::PROTOCOL_OPENPGP;
    let mut ctx = gpgme::create_context().unwrap();
    ctx.set_protocol(proto);

    let mut input = Data::from_bytes(dat).unwrap();
    let mut output = Data::new().unwrap();

    let key = ctx.find_key(keystr).unwrap();

    let result = ctx.encrypt(&[key], ops::ENCRYPT_ALWAYS_TRUST,  &mut input, &mut output);

    println!("encrypt result: {:?}", result);

    output
}



pub fn handle_1_0 (mut br: BufReader<TcpStream>, mut bw: BufWriter<TcpStream>, mut peers: Arc<Mutex<Peers>>, mut wq: Arc<Mutex<WorkQueue>>) -> Result<String,String> {
    println!("1.0....");    

    let mut headers = read_headers(&mut br).unwrap();
    println!("I got headers: {:?}", headers);
    let mut b = read_body(&mut br, &headers).unwrap().unwrap();

    println!("handle_1_0: {:?}", b);

    let response = match b.get("message") {
        None => {return Err("no 'message' field in request".to_string())}
        Some(mtype) => {
            match mtype.as_ref() {
                "ping" => {ping(&b)}
                "introduce" => {introduce(&b, wq)}
                _ => {return Err("failed to match message type".to_string())}
            }
        }
    };


    println!("handle_1_0 response: {:?}", response);

    //encode the resopnse to JSON
    let jresponse = json::encode(&response).unwrap(); 
    println!("json response: {:?}", jresponse);


    bw.write("idunn: 1.0\n".to_string().as_bytes());
    if headers.contains_key("encrypted-length") {
        //respond with encrypted data
        let jrclone = jresponse.clone();
        let key = b.get("keyid").unwrap().clone();
        let enc = encrypt(key.to_string(), &jrclone);
        let encbytes = enc.into_bytes().unwrap();
        let esize = encbytes.len();
        write!(bw, "encrypted-length: {}\n", esize);
        bw.write(&encbytes[..]);

    } else if headers.contains_key("sig-length") {
        //respond with signed data
        //TODO: this is stupid code
        //why does it need to be cloned?
        let jrclone = jresponse.clone();
        let sig = sign(&jrclone);
        let sigbytes = sig.into_bytes().unwrap();
        let dsize = jresponse.len();
        let ssize = sigbytes.len();
        write!(bw, "signed-length: {}\n", ssize);
        write!(bw, "data-length: {}\n", dsize);
        bw.write(&jresponse.into_bytes()[..]);
        bw.write(&sigbytes[..]);
    }


    Err("impossible 1.0".to_string())
}


pub fn ping(hin: &HashMap<String, String>) -> HashMap<String, String> {
    let mut hout = HashMap::new();
    hout.insert("message".to_string(), "pong".to_string());
    hout
}

pub fn introduce(hin: &HashMap<String, String>,
                 wq: Arc<Mutex<WorkQueue>>) -> HashMap<String, String> {
    print!("introducing...");
    let mut hout = HashMap::new();
    let k = hin.get("keyid").unwrap();
    let a = hin.get("address").unwrap();
    println!("{} @ {}", k, a);
    let mut wq1 = wq.lock().unwrap();
//    let &mut q: Vec<IdunnTask> = wq1.queue;
    println!("got the queue lock");
    let it = IdunnTask::Introduce{keyid: k.clone(), address: a.clone()};
    println!("going to insert {:?}", it);
    wq1.insert(it);
    println!("server sees in queue: {:?}",wq1.queue);
 

    // check for keyid in GPG

    // insert an outgoing ping into the work queue
    
    hout
}

