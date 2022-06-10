use std::{net::{TcpListener}, io::{Read, Cursor}, result::{Result}};
use anyhow::Context;
use byteorder::ReadBytesExt;
use crate::protocol::structs::{Readable, Writeable, VarInt, ProtocolVersion};

mod protocol;
mod util;

const PROTOCOL: ProtocolVersion = ProtocolVersion::V1_18_2;

struct Config {
    port: u16,
}

fn main() {
    println!("Hello, minecraft!");
    let tcp = TcpListener::bind("127.0.0.1:25565").expect("Unable to bind to port! Is another process running?");
    
    for connection in tcp.incoming() {
        let connection = connection.expect("Unable to read TCP connection!");
        
    }
}
