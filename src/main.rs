use std::{net::TcpListener, io::Read};


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
    let tcp: TcpListener;
    //assign it a value here. This keeps it way more concise.
    if let Ok(result) = TcpListener::bind(format!("127.0.0.1:{}", config.port)) {
        tcp = result;
        println!("Server bound to \"127.0.0.1:{}\"", config.port);
    } else {
        println!("Unable to bind to port {}, is another server already bound to it?", config.port);
        return
    }

    for result in tcp.incoming() {
        if let Ok(mut connection) = result {
            let mut buf = [0; 128];
            
            connection.read(&mut buf).unwrap();

            let mut size: u8 = 0;
            let mut int_vec: [u8; 4]  = [0; 4];
            
            for (i, byte) in buf.iter().enumerate() {
                // match i {
                //     0 => size = buf[i],
                //     1..5 => int_vec[i] = byte.clone(),
                //     _ => {}
                // }
                if i == 0 {
                    size = byte.clone()
                } else if i >= 1 && i<5 {
                    int_vec[i-1] = byte.clone()
                }
            }

            println!("Whole buf: {buf:?}");
            println!("Useful size: {size}" );
            println!("Protocol version: {}", as_u32(&int_vec));
        }
    }

}

fn as_u32(array: &[u8; 4]) -> u32 {
    ((array[0] as u32) << 0) + 
    ((array[1] as u32) << 8) + 
    ((array[2] as u32) << 16) + 
    ((array[3] as u32) << 24) 
}
