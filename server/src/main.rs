extern crate mio;
extern crate doosknet;

use std::collections::{HashMap, VecDeque};
use std::io::{self, Read};
use std::str;
use std::net::ToSocketAddrs;
use mio::*;
use mio::net::{TcpListener, TcpStream};
use doosknet::*;

static SERVER_USERNAME: &'static str = "SERVER";

struct Connection {
    token: Token,
    socket: TcpStream,
    is_disconnected: bool,
    buffer: NetworkBuffer,
    outgoing_packets: VecDeque<Packet>
}

impl Connection {
    pub fn new(token: Token, socket: TcpStream) -> Self {
        Connection {
            token,
            socket,
            is_disconnected: false,
            buffer: NetworkBuffer::new(),
            outgoing_packets: VecDeque::new()
        }
    }
}

fn main() {
    // Setup the server socket
    let addr = "0.0.0.0:7667".parse().unwrap();
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

    let mut incoming_packets: VecDeque<Packet> = VecDeque::new();

    loop {
        poll.poll(&mut events, None).unwrap();

        for event in events.iter() {
            match event.token() {
                LOCAL_TOKEN => {
                    match server.accept() {
                        Ok((socket, addr)) => {
                            println!("New connection from {}", addr);

                            next_token_index += 1;
                            let token = Token(next_token_index);

                            poll.register(&socket, token, Ready::readable() | Ready::writable(), PollOpt::edge()).unwrap();

                            send_all_msg("A client entered the room.", &mut connections);

                            let mut connection = Connection::new(token, socket);
                            send_msg("Welcome to Rustychat!", &mut connection);
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
                            match connection.socket.read(&mut connection.buffer.data) {
                                Ok(0) => {
                                    // Socket is closed
                                    println!("Client {:?} has disconnected!", token);
                                    connection.is_disconnected = true;

                                    break;
                                },
                                Ok(read_bytes) => {
                                    connection.buffer.offset += read_bytes;
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
                        // Send all outgoing packets
                        while let Some(packet) = connection.outgoing_packets.pop_front() {
                            let data = serialize_packet(packet);
                            match send_bytes(&mut connection.socket, &data) {
                                Ok(sent_bytes) => {
                                    println!("Sent {} bytes to client {:?}", sent_bytes, connection.token);
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
        }

        // Remove any disconnected clients
        connections = connections.into_iter()
            .filter(|&(_, ref v)| !v.is_disconnected)
            .collect();

        // Process incoming bytes to create packets
        for (_, connection) in &mut connections {
            poll.reregister(&connection.socket, connection.token, Ready::readable() | Ready::writable(), PollOpt::edge()).unwrap();

            if connection.buffer.offset == 0 {
                continue;
            }

            while let Some(packet) = deserialize_packet(&mut connection.buffer) {
                incoming_packets.push_back(packet);
            }

            connection.buffer.clear();
        }

        // Handle packets
        while let Some(packet) = incoming_packets.pop_front() {
            println!("> {}", packet.message);

            send_all(packet, &mut connections);
        }
    }
}

fn send(packet: Packet, connection: &mut Connection) {
    connection.outgoing_packets.push_back(packet.clone());
}

fn send_all(packet: Packet, connections: &mut HashMap<Token, Connection>) {
    for (_, connection) in connections {
        connection.outgoing_packets.push_back(packet.clone());
    }
}

fn send_msg(message: &str, connection: &mut Connection) {
    let packet: Packet = Packet::new(SERVER_USERNAME, message);
    send(packet, connection);
}

fn send_all_msg(message: &str, connections: &mut HashMap<Token, Connection>) {
    let packet: Packet = Packet::new(SERVER_USERNAME, message);
    send_all(packet, connections);
}