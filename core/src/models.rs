use std::io::Error;
use std::str;
use std::io::ErrorKind;
use crate::utils;

pub struct Header {
    pub message_type: [u8; 4],
    pub sender_id: [u8; 8],
    pub message_length: u32
}

pub struct Message {
    header: Header,
    data: Vec<u8>
}

impl Message {
    pub fn new(header: Header, data: Vec<u8>) -> Message {
        Message { header, data}
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut serialized = vec![];
        serialized.append(&mut self.header.to_bytes());
        serialized.append(&mut self.data.clone());

        serialized
    }
}

impl Header {
    pub fn new(message_type: [u8; 4], sender_id: [u8; 8], message_length: u32) -> Header {
        Header {
            message_type: message_type,
            sender_id: sender_id,
            message_length: message_length
        }
    }

    pub fn from_bytes(data: &Vec<u8>) -> Result<Header, Error> {
        if data.len() == 16 {
            let mut message_type: [u8; 4] = [0; 4];
            let mut sender_id: [u8; 8] = [0; 8];
            let mut message_length: [u8; 4] = [0; 4];
            message_type.copy_from_slice(&data[0..4]);
            sender_id.copy_from_slice(&data[4..12]);
            message_length.copy_from_slice(&data[12..16]);

            Ok(
                Header::new(message_type, sender_id, utils::as_u32_be(&message_length))
            )
        } else if data.len() == 0 {
            Err(Error::new(ErrorKind::Other, "no data to construct a header"))
        } else {
            Err(Error::new(ErrorKind::Other, "data vector is too small to construct a header"))
        }
    }

    pub fn size() -> u8 {
        16 
    }

    pub fn get_message_type_string(&self) -> Result<&str, Error> {
        match str::from_utf8(&self.message_type) {
            Ok(ok) => Ok(ok),
            Err(e) => Err(Error::new(ErrorKind::Other, e.to_string()))
        }
    }

    pub fn get_sender_id_string(&self) -> Result<&str, Error> {
        match str::from_utf8(&self.sender_id) {
            Ok(ok) => Ok(ok),
            Err(e) => Err(Error::new(ErrorKind::Other, e.to_string()))
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut serialized = vec![];
        serialized.append(&mut self.message_type.to_vec());
        serialized.append(&mut self.sender_id.to_vec());
        serialized.append(&mut utils::transform_u32_to_array_of_u8(self.message_length).to_vec());

        serialized
    }
}

pub struct Response {
    message_length: u32,
    data: Vec<u8>
}

impl Response {
    pub fn new(message_length: u32, data: Vec<u8>) -> Response {
        Response {message_length: message_length, data: data}
    }

    pub fn empty() -> Response {
        Response {message_length: 0, data: vec![]}
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut serialized = vec![];
        serialized.append(&mut utils::transform_u32_to_array_of_u8(self.message_length).to_vec());
        serialized.append(&mut self.data.clone());

        serialized
    }
}