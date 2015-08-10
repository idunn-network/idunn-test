#![allow(dead_code)]
#![allow(unused_variables)] 
#![allow(unused_imports)]
#![allow(unused_must_use)]
#![allow(unused_mut)]

use std::fs::File;
use std::io::{Read,Write};
use std::env;
use std::path::Path;
use toml;

pub fn read_config() -> Option<toml::Value>{
    let mut filename = match env::home_dir() {
        Some(pathbuf) => pathbuf,
        None => env::current_dir().unwrap()
    };
    filename.push(".idunn.toml");
    let mut conffile: File =  File::open(filename).unwrap();
    let mut conf: String = String::new();
    let _ = conffile.read_to_string(&mut conf);
    println!("read conf: {:?}", conf);
    let tconf: toml::Value = conf.parse().unwrap();

    println!("docs: {:?}", tconf);
    tconf.lookup("foo");
//    println!("bind_addr: {:?}", tconf.lookup("network.bind_address"));
    Some(tconf)
}


#[test]
fn read_yaml_test() {
    read_config();
}
