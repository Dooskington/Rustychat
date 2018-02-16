extern crate tokio;
extern crate tokio_io;
extern crate futures;
extern crate bytes;

use std::str;
use tokio::executor::current_thread;
use tokio::net::TcpListener;
use tokio_io::{io, AsyncRead};
use futures::{Future, Stream};
use bytes::{BytesMut, BufMut};

fn main() {
    let addr = "127.0.0.1:7667".parse().unwrap();
    let listener = TcpListener::bind(&addr).unwrap();

    let server = listener.incoming().for_each(|socket| {
        let remote_addr = socket.peer_addr().unwrap();
        println!("Connection accepted: {}", remote_addr);

        let (reader, writer) = socket.split();

        /*
        let welcome = io::write_all(writer, "Welcome to the server.\n")
            .then(move |result| {
                match result {
                    Ok((_, buf)) => println!("Wrote {} bytes to {}", buf.len(), remote_addr),
                    Err(e) => println!("Error on {}: {}", remote_addr, e),
                }
                Ok(())
            });
        */

        let mut buf = BytesMut::with_capacity(1024);
        buf.reserve(2);
        buf.put_slice(b"xy");
        println!("len is {}, contents are {}", buf.len(), str::from_utf8(&buf).unwrap());
        let read_input = io::read(reader, buf)
            .then(move |result| {
                match result {
                    Ok((_, buf, size)) => println!("{} says {}", remote_addr, size),
                    Err(e) => println!("Error on {}: {}", remote_addr, e),
                }
                Ok(())
            });

        /*
        let echo = io::copy(reader, writer);

        let complete = echo.then(move |result| {
            match result {
                Ok((sent, _, _)) => println!("Wrote {} bytes to {}", sent, remote_addr),
                Err(e) => println!("Error on {}: {}", remote_addr, e),
            }

            Ok(())
        });
        */

        // Spawn a new task that handles the socket:
        current_thread::spawn(read_input);

        Ok(())
    })
    .map_err(|err| {
        println!("Error! {:?}", err);
    });

    current_thread::run(|_| {
        current_thread::spawn(server);

        println!("Server running on {}", addr);
    });
}


// io::write_all
/*
    "The buf parameter here only requires the AsRef<[u8]> trait,
    which should be broadly applicable to accepting data which can be converted to a slice."
    Could be useful? Maybe packets implement that trait.
*/