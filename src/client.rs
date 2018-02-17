extern crate mio;
extern crate bytes;
extern crate gfx;

use std::time::Duration;
use std::collections::VecDeque;
use std::io::{self, Read, Write, Error, ErrorKind};
use std::str;
use mio::*;
use mio::net::TcpStream;
use gfx::{input, Window, Renderer};
use gfx::input::{InputMan};

const LOCAL_TOKEN: Token = Token(0);

fn main() {
    let window_title: &str = "Rustychat";
    let window_width: u32 = 50 * gfx::CELL_WIDTH;
    let window_height: u32 = 15 * gfx::CELL_HEIGHT;

    let mut window: Window = Window::new(window_title, window_width, window_height);
    let mut renderer: Renderer = Renderer::new(&window);
    let mut input_man: InputMan = InputMan::new();

    // Setup the client socket
    let addr = "127.0.0.1:7667".parse().unwrap();
    let mut socket = TcpStream::connect(&addr).unwrap();

    // Create a poll instance
    let poll = Poll::new().unwrap();

    // Register the socket
    poll.register(&socket, LOCAL_TOKEN, Ready::readable() | Ready::writable(), PollOpt::edge()).unwrap();

    // Create storage for events
    let mut events = Events::with_capacity(1024);

    let mut buffer = [0; 1024];

    let mut is_disconnected: bool = false;

    let mut outgoing_packets: VecDeque<String> = VecDeque::new();

    let mut messages: Vec<String> = Vec::new();
    messages.push(String::from("one"));
    messages.push(String::from("two"));
    messages.push(String::from("three"));

    loop {
        // UI
        input::process_events(&mut window, &mut input_man);
        if window.is_close_requested {
            break;
        }

        if input::is_key_pressed(&input_man, input::VirtualKeyCode::Return) {
            let message: String = input_man.input_string.clone();
            input_man.clear_input_string();

            outgoing_packets.push_back(message);
        }

        gfx::clear(&mut renderer);

        let mut line_count = 0;
        for message in &messages {
            if line_count >= 32 {
                break;
            }

            gfx::draw_string(&mut renderer, 0, 1 + line_count, &message);
            line_count += 1;
        }

        gfx::draw_string(&mut renderer, 0, 0, &format!("> {}", input_man.input_string));

        gfx::render(&mut renderer);
        gfx::display(&window);

        input::update_input(&mut input_man);

        // Networking
        poll.poll(&mut events, Some(Duration::from_millis(1))).unwrap();
        for event in events.iter() {
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
                            match send_bytes(&mut socket, packet.as_bytes()) {
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

        // Need to reregister for events
        poll.reregister(&socket, LOCAL_TOKEN, Ready::readable() | Ready::writable(), PollOpt::edge()).unwrap();
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