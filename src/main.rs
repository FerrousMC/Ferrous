use std::{net::{TcpListener}, io::{Read, Cursor}, result::{Result}};
use byteorder::ReadBytesExt;
use crate::protocol::structs::{Readable, Writeable, VarInt, ProtocolVersion};

mod protocol;
mod world;

const PROTOCOL: ProtocolVersion = ProtocolVersion::V1_18_2;

struct Config {
    port: u16,
}

fn main() {
    println!("Hello, world!");

    //eventually, this will be read from config
    let config = Config {
        port: 25565,
    };

    //define our tcp listener here

    let addr = "127.0.0.1:25565";
    let tcp: TcpListener = TcpListener::bind(addr).expect(&format!("Couldn't bind to address \"{}\"! Is another process bound to it?", &addr));
    for result in tcp.incoming() {
        if let Ok(mut connection) = result {
            let mut buf = [0; 256];
            connection.read(&mut buf).unwrap();
            let mut cursor: Cursor<&[u8]> = Cursor::new(&buf);

            let version: i32 = VarInt::read(&mut cursor, PROTOCOL).unwrap().0;
            let address: String = String::read(&mut cursor, PROTOCOL).unwrap();
            let port: u8 = u8::read(&mut cursor, PROTOCOL).unwrap();
            let next_state: i32 = VarInt::read(&mut cursor, PROTOCOL).unwrap().0;

            println!("Version: {}", version);
            println!("Address: {}", address);
            println!("Port: {}", port);
            println!("Next state: {} => {}", next_state, if next_state == 1 {"1"} else if next_state == 2 {"2"} else {"?"});
        } else {
            println!("Recieved a bad connection!")
        }
    }

}

// [16, 0, 246, 5, 9, 108, 111, 99, 97, 108, 104, 111, 115, 116, 99, 221, 2,
// [16, 0, 246, 5, 9, 108, 111, 99, 97, 108, 104, 111, 115, 116, 99, 221, 2, 7, 0, 5, 115, 48, 118, 105, 95, 0]
