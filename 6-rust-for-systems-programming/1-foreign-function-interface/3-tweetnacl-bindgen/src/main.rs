use std::mem::MaybeUninit;

fn main() {
    println!("{:?}", crypto_hash_sha512_tweet("Hello, world!".as_bytes()));
}

pub fn crypto_hash_sha512_tweet(data: &[u8]) -> [u8; 64] {
    let mut ret: MaybeUninit<[u8; 64]> = MaybeUninit::uninit();
    unsafe {
        tweetnacl_bindgen::bindings::crypto_hash_sha512_tweet(
            ret.as_mut_ptr() as *mut u8,
            data.as_ptr(),
            data.len() as _,
        );
        ret.assume_init()
    }
}

pub fn crypto_hash_sha256_tweet(_data: &[u8]) -> [u8; 64] {
    // TODO: crypto_hash_sha256_tweet not found in tweetnacl.c.
    todo!()
}
