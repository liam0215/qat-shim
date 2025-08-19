use std::sync::mpsc::{TryRecvError, channel};

use serial_test::serial;

use qat_shim::qat::{
    self, DATA_LEN, Instance, Status, hash_sha512, qae_mem_destroy, qae_mem_init, start_session,
    stop_session,
};

pub fn sign_test_message(inst: &Instance) -> Result<(), Status> {
    // RFC 8032 Ed25519 test private key
    let private_key: [u8; DATA_LEN] = [
        0x83, 0x3f, 0xe6, 0x24, 0x09, 0x23, 0x7b, 0x9d, 0x62, 0xec, 0x77, 0x58, 0x75, 0x20, 0x91,
        0x1e, 0x9a, 0x75, 0x9c, 0xec, 0x1d, 0x19, 0x75, 0x5b, 0x7d, 0xa9, 0x01, 0xb9, 0x6d, 0xca,
        0x3d, 0x42,
    ];

    let message = "abc";
    let hash = hash_sha512(message).expect("hash_sha512 failed");

    println!("Message Hash: {:?}", hash);
    let public_key = if inst.is_polled()? {
        let (tx, rx) = channel();
        let inst2 = inst.clone();
        let poll = std::thread::spawn(move || {
            while matches!(rx.try_recv(), Err(TryRecvError::Empty)) {
                let _ = inst2.clone().poll_once();
            }
        });
        let public_key = inst.eddsa_gen_public_key(&private_key)?;
        tx.send(())
            .expect("Failed to send stop signal to polling thread");
        poll.join().expect("Polling thread panicked");
        public_key
    } else {
        inst.eddsa_gen_public_key(&private_key)?
    };

    println!("Public Key: {:?}", public_key);

    let signature = if inst.is_polled()? {
        let (tx, rx) = channel();
        let inst2 = inst.clone();
        let poll = std::thread::spawn(move || {
            while matches!(rx.try_recv(), Err(TryRecvError::Empty)) {
                let _ = inst2.clone().poll_once();
            }
        });
        let signature = inst.eddsa_sign_msg(&private_key, &hash)?;
        tx.send(())
            .expect("Failed to send stop signal to polling thread");
        poll.join().expect("Polling thread panicked");
        signature
    } else {
        inst.eddsa_sign_msg(&private_key, &hash)?
    };

    println!("Signature: {:?}", signature);

    // Verify the signature using the public key
    let is_valid = if inst.is_polled()? {
        let (tx, rx) = channel();
        let inst2 = inst.clone();
        let poll = std::thread::spawn(move || {
            while matches!(rx.try_recv(), Err(TryRecvError::Empty)) {
                let _ = inst2.clone().poll_once();
            }
        });
        let is_valid = inst.eddsa_verify_msg(&public_key, &hash, &signature);
        tx.send(())
            .expect("Failed to send stop signal to polling thread");
        poll.join().expect("Polling thread panicked");
        is_valid
    } else {
        inst.eddsa_verify_msg(&public_key, &hash, &signature)
    };

    is_valid
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
    sign_test_message(&inst).expect("sign test message failed");
    inst.stop().expect("stop instance failed");
    stop_session().expect("stop session failed");
    qae_mem_destroy();
}
