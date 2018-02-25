extern crate libc;

#[derive(Debug)]
pub enum ShaError {
    InvalidArg
}

#[link(name = "sha256")]
extern {
    fn sha256_init_digest(digest: *mut u32);
    fn sha256(input_data: *const u8, digest: *mut u32, num_blocks: u32);
    fn sha256_iterate(input_data: *mut u8, digest: *const u32, num_iterations: i64);
}

pub fn hash_iterate256(input_data: &mut[u8], digest: &[u32], num_iterations: i64) -> Result<i32, ShaError> {
    if input_data.len() == 0 || (input_data.len() % 64) != 0 {
        println!("invalid input len: {}", input_data.len());
        return Err(ShaError::InvalidArg);
    }
    unsafe {
        sha256_iterate(input_data.as_mut_ptr(), digest.as_ptr(), num_iterations);
    }
    Ok(0)
}

#[inline]
pub fn compress256(state: &mut[u32; 8], input_data: &[u8]) -> Result<i32, ShaError> {
    if input_data.len() == 0 || (input_data.len() % 64) != 0 {
        println!("invalid input len: {}", input_data.len());
        return Err(ShaError::InvalidArg);
    }
    unsafe {
        sha256(input_data.as_ptr(), state.as_mut_ptr(), (input_data.len() / 64) as u32);
    }
    Ok(0)
}

pub fn init_digest(digest: &mut[u32; 8]) -> Result<i32, ShaError> {
    unsafe {
        sha256_init_digest(digest.as_mut_ptr());
    }
    Ok(0)
}



#[cfg(test)]
mod tests {
    use std;
    use sha256;

    struct ByteBuf<'a>(&'a [u8]);
    struct U32Buf<'a>(&'a [u32]);

    impl<'a> std::fmt::LowerHex for ByteBuf<'a> {
        fn fmt(&self, fmtr: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
            for byte in self.0 {
                try!( fmtr.write_fmt(format_args!("0x{:02x}, ", byte)));
            }
            Ok(())
        }
    }

    impl<'a> std::fmt::LowerHex for U32Buf<'a> {
        fn fmt(&self, fmtr: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
            for byte in self.0 {
                try!( fmtr.write_fmt(format_args!("0x{:02x}, ", byte)));
            }
            Ok(())
        }
    }

    #[test]
    fn sha256_test() {
        let input_data = "AnatolyYakovenko11/2/201712pmPSTAnatolyYakovenko11/2/201712pmPST".as_bytes();
        let mut mut_input_data = input_data.to_vec();
        let mut digest : [u32 ; 8] = [0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7];
        println!("input: {:x}\n", ByteBuf(&input_data));

        let res = sha256::init_digest(&mut digest);
        assert!(res.is_ok());
        assert_eq!(digest, [0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19]);
 
        println!("{:x} {:?}\n", U32Buf(&digest), res);
        let res = sha256::compress256(&mut digest, &input_data);
        println!("{:x} {:?}\n", U32Buf(&digest), res);
        assert!(res.is_ok());
        assert_eq!(digest, [0x88562e6c, 0x6611c0dd, 0x204b4616, 0xd72a2299, 0xa266cce9, 0xce2eec35, 0x1cf5b630, 0x814314ba]);

        let res = sha256::hash_iterate256(mut_input_data.as_mut_slice(), &digest, 100);
        println!("iterate: {:x} {:?}\n", ByteBuf(&mut_input_data), res);
    }
}

