#[macro_use]
extern crate serde_derive;

extern crate mio;
extern crate serde;
extern crate bincode;
extern crate byteorder;

use std::io::{self, Write, Error, ErrorKind, Cursor};
use std::str;
use mio::*;
use mio::net::TcpStream;
use byteorder::{NetworkEndian, ReadBytesExt, WriteBytesExt};

pub const LOCAL_TOKEN: Token = Token(0);
pub const MAX_BUFFER_SIZE: usize = 1024;
pub const PACKET_HEADER_SIZE: usize = 8;
pub const MAX_PACKET_BODY_SIZE: usize = 256;
pub const MAX_PACKET_SIZE: usize = PACKET_HEADER_SIZE + MAX_PACKET_BODY_SIZE;

#[derive(Serialize, Deserialize, Clone)]
pub struct Packet {
    pub sender: String,
    pub message: String
}

impl Packet {
    pub fn new(sender: &str, message: &str) -> Self {
        let sender = String::from(sender);
        let message = String::from(message);

        Packet {
            sender,
            message
        }
    }
}

pub fn serialize_packet(packet: Packet) -> Vec<u8> {
    // Body
    let mut body_data: Vec<u8> = bincode::serialize(&packet).unwrap();

    // Header
    let mut data: Vec<u8> = Vec::new();
    data.write_u64::<NetworkEndian>(body_data.len() as u64).unwrap();

    // Combine the body and header
    data.append(&mut body_data);

    data
}

pub fn deserialize_packet(buffer: &mut NetworkBuffer) -> Option<Packet> {
    // Ensure there is enough data for a packet header
    if buffer.offset < PACKET_HEADER_SIZE {
        return None;
    }

    let body_size: usize;
    let packet: Option<Packet>;
    {
        let mut reader = Cursor::new(&buffer.data[..]);

        // Read header
        body_size = reader.read_u64::<NetworkEndian>().unwrap() as usize;
        if body_size >= MAX_PACKET_BODY_SIZE {
            eprintln!("Packet body too large! {} >= {}", body_size, MAX_PACKET_BODY_SIZE);
            return None;
        }

        // Ensure there is enough data for the rest of the packet
        if buffer.offset < (body_size + PACKET_HEADER_SIZE) {
            return None;
        }

        let deserialized: Packet = bincode::deserialize(&buffer.data[PACKET_HEADER_SIZE..]).unwrap();

        packet = Some(deserialized);
    }

    buffer.drain(body_size + PACKET_HEADER_SIZE);

    packet
}

pub struct NetworkBuffer {
    pub data: [u8; MAX_BUFFER_SIZE],
    pub offset: usize
}

impl NetworkBuffer {
    pub fn new() -> Self {
        NetworkBuffer {
            data: [0; MAX_BUFFER_SIZE],
            offset: 0
        }
    }

    pub fn drain(&mut self, count: usize) {
        unsafe {
            use std::ptr;
            ptr::copy(self.data.as_ptr().offset(count as isize), self.data.as_mut_ptr(), self.offset - count);
        }

        self.offset -= count;
    }

    pub fn clear(&mut self) {
        self.data = [0; MAX_BUFFER_SIZE];
        self.offset = 0;
    }
}

pub fn send_bytes(socket: &mut TcpStream, buffer: &[u8]) -> Result<usize, io::Error> {
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