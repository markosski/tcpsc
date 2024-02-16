use std::{fmt, str};
use std::str::FromStr;
use std::io::BufReader;
use std::io::Read;

use crate::utils::{self, GeneralError};
use utils::Result;

pub struct Header {
    pub message_type: [u8; 4],
    pub sender_id: [u8; 8],
    pub message_length: u32
}

pub struct Message {
    pub header: Header,
    pub data: Vec<u8>
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

    pub fn from_bytes(data: &Vec<u8>) -> Result<Header> {
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
            Err(GeneralError::new( "no data to construct a header".to_string()))
        } else {
            Err(GeneralError::new( "data vector is too small to construct a header".to_string()))
        }
    }

    pub fn size() -> u8 {
        16 
    }

    pub fn get_message_type_string(&self) -> Result<&str> {
        match str::from_utf8(&self.message_type) {
            Ok(ok) => Ok(ok),
            Err(e) => Err(GeneralError::new(format!("{}", e)))
        }
    }

    pub fn get_sender_id_string(&self) -> Result<&str> {
        match str::from_utf8(&self.sender_id) {
            Ok(ok) => Ok(ok),
            Err(e) => Err(GeneralError::new(format!("{}", e)))
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

#[derive(Debug, PartialEq)]
pub enum ResponseType {
    SUCC, ERR 
}

impl ResponseType {
    fn to_bytes(&self) -> Vec<u8> {
        match self {
            ResponseType::SUCC => "SUCC".as_bytes().to_vec(),
            ResponseType::ERR => "ERR ".as_bytes().to_vec()
        }
    }
}

impl fmt::Display for ResponseType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl FromStr for ResponseType {
    type Err = Box<dyn std::error::Error>;

    fn from_str(input: &str) -> std::result::Result<ResponseType, Self::Err> {
        match input {
            "SUCC"  => Ok(ResponseType::SUCC),
            "ERR "  => Ok(ResponseType::ERR),
            _      => Err(GeneralError::new(format!("Could not create response type for name {}", &input))),
        }
    }
}


#[derive(Debug, PartialEq)]
pub struct Response {
    pub response_type: ResponseType,
    message_length: u32,
    pub data: Vec<u8>
}

impl ToString for Response {
    fn to_string(&self) -> String {
        let data_as_string = str::from_utf8(&self.data[..]).unwrap();
        format!("{} {}", &self.response_type, &data_as_string)
    }
}

impl Response {
    pub fn success(data: Vec<u8>) -> Response {
        Response {response_type: ResponseType::SUCC, message_length: data.len() as u32, data: data}
    }

    pub fn empty() -> Response {
        Response {response_type: ResponseType::SUCC, message_length: 0, data: vec![]}
    }

    pub fn error(error_message: String) -> Response {
        Response {response_type: ResponseType::ERR, message_length: error_message.len() as u32, data: error_message.into_bytes()}
    }

    pub fn as_string(&self) -> (Option<&str>, Option<&str>) {
        match self.response_type {
            ResponseType::SUCC => (Some(str::from_utf8(&self.data[..]).unwrap()), None),
            ResponseType::ERR => (None, Some(str::from_utf8(&self.data[..]).unwrap()))
        }
    }

    pub fn success_as_string(&self) -> Option<&str> {
        match self.response_type {
            ResponseType::SUCC => Some(str::from_utf8(&self.data[..]).unwrap()),
            ResponseType::ERR => None
        }
    }

    pub fn error_as_string(&self) -> Option<&str> {
        match self.response_type {
            ResponseType::ERR => Some(str::from_utf8(&self.data[..]).unwrap()),
            ResponseType::SUCC => None
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut serialized = vec![];
        serialized.append(&mut self.response_type.to_bytes());
        serialized.append(&mut utils::transform_u32_to_array_of_u8(self.message_length).to_vec());
        serialized.append(&mut self.data.clone());

        serialized
    }

    pub fn from_buffer<T: Read>(buf: &mut BufReader<&mut T>) -> Result<Response> {
        let mut response_type: [u8; 4] = [0; 4];
        buf.read_exact(&mut response_type).unwrap();
        let message_type_utf = str::from_utf8(&response_type)?;
        let response_type = ResponseType::from_str(message_type_utf);

        let mut response_length_bytes: [u8; 4] = [0; 4];
        buf.read_exact(&mut response_length_bytes)?;
        let response_length = utils::as_u32_be(&response_length_bytes) as u64;

        let mut body: Vec<u8> = vec![];
        buf.take(response_length).read_to_end(&mut body)?;

        match response_type {
            Ok(ResponseType::SUCC) => Ok(Response::success(body)),
            Ok(ResponseType::ERR) => {
                let message_data_utf = str::from_utf8(&body)?;
                Ok(Response::error(message_data_utf.to_string()))
            },
            Err(_) => panic!("did not recognize response")
        }
    }

    pub fn from_bytes(data: &Vec<u8>) -> Result<Response> {
        if data.len() >= 8 {
            let mut message_type: [u8; 4] = [0; 4];
            let mut message_length: [u8; 4] = [0; 4];
            let message_data = data[8..].to_vec();
            message_type.copy_from_slice(&data[0..4]);
            message_length.copy_from_slice(&data[4..8]);

            let message_type_utf = str::from_utf8(&message_type).unwrap(); //TODO convert to Error
            let response_type = ResponseType::from_str(message_type_utf);

            match response_type {
                Ok(ResponseType::SUCC) => Ok(Response::success(message_data)),
                Ok(ResponseType::ERR) => {
                    let message_data_utf = str::from_utf8(&message_data).unwrap(); //TODO convert to Error
                    Ok(Response::error(message_data_utf.to_string()))
                },
                Err(_) => panic!("did not recognize response")
            }
        } else {
            Err(GeneralError::new("data vector is too small to construct a response".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn response_success_serde() {
        let response = Response::success("hello world".as_bytes().to_vec());
        let resp_bytes = response.to_bytes();
        let response_result = Response::from_bytes(&resp_bytes);

        assert_eq!(response.data, response_result.unwrap().data);
    }

    #[test]
    fn response_error_serde() {
        let response = Response::error("this is an error".to_string());
        let resp_bytes = response.to_bytes();
        let response_result = Response::from_bytes(&resp_bytes).unwrap();

        println!("left: {:?}, right: {:?}", &response, &response_result);
        assert_eq!(response.data, response_result.data);
    }
}