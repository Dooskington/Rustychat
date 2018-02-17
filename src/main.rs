extern crate mio;
extern crate bytes;

use std::collections::{HashMap, VecDeque};
use std::io::{self, Read, Write, Error, ErrorKind};
use std::str;
use mio::*;
use mio::net::{TcpListener, TcpStream};

const LOCAL_TOKEN: Token = Token(0);

struct Connection {
    token: Token,
    socket: TcpStream,
    is_disconnected: bool,
    buffer: [u8; 1024],
    buffer_offset: usize,
    outgoing_packets: VecDeque<String>
}

impl Connection {
    pub fn new(token: Token, socket: TcpStream) -> Self {
        Connection {
            token,
            socket,
            is_disconnected: false,
            buffer: [0; 1024],
            buffer_offset: 0,
            outgoing_packets: VecDeque::new()
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

    let mut incoming_packets: VecDeque<String> = VecDeque::new();

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
                            match connection.socket.read(&mut connection.buffer) {
                                Ok(0) => {
                                    // Socket is closed
                                    println!("Client {:?} has disconnected!", token);
                                    connection.is_disconnected = true;

                                    break;
                                },
                                Ok(read_bytes) => {
                                    connection.buffer_offset += read_bytes;
                                    println!("Read {} bytes from client {:?}, they have {} bytes so far", read_bytes, token, connection.buffer_offset);
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

                    }
                }
            }
        }

        // Remove any disconnected clients
        connections = connections.into_iter()
            .filter(|&(_, ref v)| !v.is_disconnected)
            .collect();

        // Process incoming bytes to create packets
        for (_, connection) in &mut connections {
            if connection.buffer_offset == 0 {
                continue;
            }

            let len = connection.buffer_offset;
            let message = String::from(str::from_utf8(&connection.buffer[0..len]).unwrap());
            incoming_packets.push_back(message);

            connection.buffer = [0; 1024];
            connection.buffer_offset = 0;
        }

        // Handle packets
        while let Some(packet) = incoming_packets.pop_front() {
            println!("> {}", packet);

            for (_, connection) in &mut connections {
                match send_bytes(&mut connection.socket, packet.as_bytes()) {
                    Ok(sent_bytes) => {
                        println!("Sent {} bytes", sent_bytes);
                    },
                    Err(e) => {
                        eprintln!("send_bytes() failed with error {:?}", e);
                        break;
                    }
                }
            }
        }
    }
}

fn send_bytes(socket: &mut TcpStream, buffer: &[u8]) -> Result<usize, io::Error> {
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