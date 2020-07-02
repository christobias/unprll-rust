pub fn serialize(mut n: u64) -> Vec<u8> {
    let mut vec = Vec::new();

    while n > 127 {
        vec.push(128 | n as u8);
        n >>= 7;
    }

    vec.push(n as u8);

    vec
}

pub fn deserialize(bytes: &[u8]) -> u64 {
    let mut n = 0;
    let mut shift = 0;

    for byte in bytes {
        n |= ((byte & 127) as u64) << shift;
        shift += 7;

        if *byte < 128 {
            break;
        }
    }

    n
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
