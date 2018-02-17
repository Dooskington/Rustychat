extern crate mio;
extern crate bytes;
#[macro_use] extern crate text_io;

use std::collections::HashMap;
use std::collections::VecDeque;
use std::io::{self, Read, Write, Error, ErrorKind};
use std::str;
use mio::*;
use mio::net::{TcpListener, TcpStream};
use bytes::{Bytes, BytesMut, Buf, BufMut};

const LOCAL_TOKEN: Token = Token(0);

fn main() {
    // Setup the client socket
    let addr = "127.0.0.1:7667".parse().unwrap();
    let mut socket = TcpStream::connect(&addr).unwrap();

    // Create a poll instance
    let poll = Poll::new().unwrap();

    // Register the socket
    poll.register(&socket, LOCAL_TOKEN, Ready::readable() | Ready::writable(), PollOpt::edge()).unwrap();

    // Create storage for events
    let mut events = Events::with_capacity(1024);

    //let mut buffer = BytesMut::with_capacity(1024);
    //buffer.put(&b"1234_56789"[..]);
    //let mut buffer = Vec::with_capacity(1024);
    let mut buffer = [0; 1024];

    let mut is_disconnected: bool = false;

    let mut test_buf = BytesMut::with_capacity(1024);
    test_buf.put(&b"Hello from the client"[..]);

    let mut outgoing_packets: VecDeque<String> = VecDeque::new();

    loop {
        println!("Type message:");
        let input: String = read!("{}\n");
        outgoing_packets.push_back(input);

        poll.poll(&mut events, None).unwrap();
        for event in events.iter() {
            println!("{:?}", event);

            match event.token() {
                LOCAL_TOKEN => {
                    if event.readiness().is_readable() {
                        loop {
                            // Read until there are no more incoming bytes
                            match socket.read(&mut buffer) {
                                Ok(0) => {
                                    // Socket is closed
                                    println!("Disconnected from server!");
                                    is_disconnected = true;
                                    break;
                                },
                                Ok(read_bytes) => {
                                    println!("Read {} bytes from server: {}", read_bytes, str::from_utf8(&buffer).unwrap());
                                },
                                Err(e) => {
                                    if e.kind() == io::ErrorKind::WouldBlock {
                                        // Socket is not ready anymore, stop reading
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    else if event.readiness().is_writable() {
                        if outgoing_packets.is_empty() {
                            continue;
                        }

                        while let Some(packet) = outgoing_packets.pop_front() {
                            match send_all(&mut socket, packet.as_bytes()) {
                                Ok(sent_bytes) => {
                                    println!("Sent {} bytes", sent_bytes);
                                },
                                Err(e) => {
                                    eprintln!("send() failed with error {:?}", e);
                                    break;
                                }
                            }
                        }
                    }
                },
                _ => unreachable!()
            }
        }

        if is_disconnected {
            println!("Connection closed.");
            break;
        }
    }
}

fn send_all(socket: &mut TcpStream, buffer: &[u8]) -> Result<usize, io::Error> {
    let mut len = buffer.len();
    if len == 0 {
        return Err(Error::new(ErrorKind::InvalidData, "Buffer is empty!"));
    }

    // Keep sending until we've sent the entire buffer
    while len > 0 {
        match socket.write(buffer) {
            Ok(sent_bytes) => {
                len -= sent_bytes;
            },
            Err(e) => {
                return Err(e);
            }
        }
    }

    Ok(buffer.len())
}