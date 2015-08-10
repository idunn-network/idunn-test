#![allow(dead_code)]
#![allow(unused_variables)] 
#![allow(unused_imports)]
#![allow(unused_must_use)]
#![allow(unused_mut)]

extern crate gpgme;

use gpgme::Protocol;
use gpgme::Data;
use gpgme::ops;
use gpgme::keys;
use gpgme::context::*;

//pub fn is_key_known(keyid: String) -> bool { }


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
pub struct Decrypted {
    pub data: gpgme::Data<'static>,
    pub dr: gpgme::ops::DecryptResult
}

/*
pub fn passphrase_cb(_hint: Option<&str>, _info: Option<&str>,
                     _prev_was_bad: bool, out: &mut Write) -> gpgme::Result<()> {
    try!(out.write_all(b"abc\n"));
    Ok(())
}
*/
