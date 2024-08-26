use libc::size_t;

fn main() {
    println!("{:#x}", crc32(b"12345678"));
    // Output should be: 0x9ae0daaf
}

extern "C" {
    fn CRC32(data: *const u8, data_length: size_t) -> u32;
}

fn crc32(data: &[u8]) -> u32 {
    unsafe { CRC32(data.as_ptr(), data.len()) }
}
