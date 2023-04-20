use rand::{
    distributions::{Alphanumeric, DistString},
    Rng,
};

pub fn random_bytes(len: usize) -> Vec<u8> {
    let mut output = vec![0u8; len];
    rand::thread_rng().fill(&mut output[..]);
    output
}

pub fn random_string(len: usize) -> String {
    Alphanumeric.sample_string(&mut rand::thread_rng(), len)
}
