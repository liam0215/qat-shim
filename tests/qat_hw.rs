use serial_test::serial;

use qat_shim::qat::{
    self, Instance, Status, hash_sha512, qae_mem_destroy, qae_mem_init, start_session, stop_session,
};

const HASH_LEN: usize = 64; // SHA-512 output size (bytes)
// const DATA_LEN: usize = 32; // Ed25519 key size (bytes)

pub fn hash_demo() -> Result<[u8; HASH_LEN], Status> {
    // Buffers (match the C code)
    // let mut message_hash = [0u8; HASH_LEN]; // messageHash
    // let mut _public_key = [0u8; DATA_LEN]; // publicKey (unused here)
    // let mut _signature = [0u8; DATA_LEN * 2]; // signature  (unused here)

    // RFC 8032 Ed25519 test private key (from your snippet)
    // let _private_key: [u8; DATA_LEN] = [
    //     0x83, 0x3f, 0xe6, 0x24, 0x09, 0x23, 0x7b, 0x9d, 0x62, 0xec, 0x77, 0x58, 0x75, 0x20, 0x91,
    //     0x1e, 0x9a, 0x75, 0x9c, 0xec, 0x1d, 0x19, 0x75, 0x5b, 0x7d, 0xa9, 0x01, 0xb9, 0x6d, 0xca,
    //     0x3d, 0x42,
    // ];

    // "abc" (no NUL required; we pass pointer + length)
    let message = "abc";

    // Hash message → message_hash
    let hash = hash_sha512(message).expect("hash_sha512 failed");
    Ok(hash)
}

#[test]
#[serial]
fn start_then_stop_first_instance() {
    start_session("SSL").expect("start session failed");
    qae_mem_init().expect("qae_mem_init failed");
    let inst: Instance = qat::get_first_instance().expect("failed to get first instance");
    inst.set_address_translation()
        .expect("set address translation failed");
    inst.start().expect("start instance failed");
    let hash = hash_demo().expect("hash demo failed");
    println!("Hash: {:?}", hash);
    inst.stop().expect("stop instance failed");
    stop_session().expect("stop session failed");
    qae_mem_destroy();
}
