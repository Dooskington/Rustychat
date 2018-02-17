extern crate mio;
extern crate bytes;

use std::collections::HashMap;
use std::io::{self, Read, Write};
use mio::*;
use mio::net::{TcpListener, TcpStream};
use bytes::{Bytes, BytesMut, Buf, BufMut};

const LOCAL_TOKEN: Token = Token(0);

struct Connection {
    token: Token,
    socket: TcpStream,
    is_disconnected: bool
}

impl Connection {
    pub fn new(token: Token, socket: TcpStream) -> Self {
        Connection {
            token,
            socket,
            is_disconnected: false
        }
    }
}

fn main() {
    // Setup the server socket
    let addr = "127.0.0.1:7667".parse().unwrap();
    let server = TcpListener::bind(&addr).unwrap();

    println!("Server started on {}", addr);

    // Create a poll instance
    let poll = Poll::new().unwrap();

    // Start listening for incoming connections
    poll.register(&server, LOCAL_TOKEN, Ready::readable(), PollOpt::edge()).unwrap();

    // Create storage for events
    let mut events = Events::with_capacity(1024);

    let mut next_token_index: usize = 0;
    let mut connections: HashMap<Token, Connection> = HashMap::new();

    let mut buffer = [0; 1024];

    let mut test_buf = BytesMut::with_capacity(1024);
    test_buf.put(&b"Test"[..]);

    loop {
        poll.poll(&mut events, None).unwrap();

        for event in events.iter() {
            println!("{:?}", event);

            match event.token() {
                LOCAL_TOKEN => {
                    match server.accept() {
                        Ok((socket, addr)) => {
                            println!("New connection from {}", addr);

                            next_token_index += 1;
                            let token = Token(next_token_index);

                            poll.register(&socket, token, Ready::readable() | Ready::writable(), PollOpt::edge()).unwrap();

                            let connection = Connection::new(token, socket);
                            connections.insert(token, connection);

                            println!("There are now {} clients connected.", connections.len());
                        },
                        Err(e) => println!("{}", e)
                    }
                },
                token => {
                    // Get the connection
                    let connection: &mut Connection = connections.get_mut(&token).unwrap();

                    if event.readiness().is_readable() {
                        loop {
                            // Read until there are no more incoming bytes
                            match connection.socket.read(&mut buffer) {
                                Ok(0) => {
                                    // Socket is closed
                                    println!("Client {:?} has disconnected!", token);
                                    connection.is_disconnected = true;

                                    break;
                                },
                                Ok(read_bytes) => {
                                    println!("Read {} bytes from client {:?}", read_bytes, token);
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
                        /*
                        loop {
                            match connection.socket.write(&test_buf) {
                                Ok(sent_bytes) => {
                                    println!("Sent {} bytes to client {:?}", sent_bytes, token);
                                    break;
                                },
                                Err(e) => {
                                    if e.kind() == io::ErrorKind::WouldBlock {
                                        break;
                                    }
                                }
                            }
                        }
                        */
                    }
                }
            }
        }

        // Remove any disconnected clients
        connections = connections.into_iter()
            .filter(|&(_, ref v)| !v.is_disconnected)
            .collect();
    }
}