use std::error::Error;
use std::fmt;

pub type Result<T> = std::result::Result<T, std::boxed::Box<dyn Error>>;

#[derive(Debug)]
pub struct GeneralError {
    message: String
}

impl GeneralError {
    pub fn new(message: String) -> Box<GeneralError> {
        Box::new(GeneralError{message: message})
    }
}

impl fmt::Display for GeneralError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.message)
    }
}

impl Error for GeneralError {
    fn description(&self) -> &str {
        &self.message
    }
}

pub fn vec_as_u32_be(vec: &Vec<u8>) -> u32 {
    let mut arr: [u8; 4] = [0; 4];
    arr.copy_from_slice(&vec[0..4]);
    as_u32_be(&arr)
}

pub fn as_u32_be(array: &[u8; 4]) -> u32 {
    ((array[0] as u32) << 24) +
    ((array[1] as u32) << 16) +
    ((array[2] as u32) <<  8) +
    ((array[3] as u32) <<  0)
}

pub fn transform_u32_to_array_of_u8(x:u32) -> [u8;4] {
    let b1 : u8 = ((x >> 24) & 0xff) as u8;
    let b2 : u8 = ((x >> 16) & 0xff) as u8;
    let b3 : u8 = ((x >> 8) & 0xff) as u8;
    let b4 : u8 = (x & 0xff) as u8;
    return [b1, b2, b3, b4]
}

pub fn to_array<const N: usize>(s: &str) -> [u8; N] {
    let mut bytes = s.bytes();
    [(); N].map(|_| bytes.next().unwrap())
}