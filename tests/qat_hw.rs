use serial_test::serial;

use qat_shim::qat::{self, Instance, qae_mem_destroy, qae_mem_init, start_session, stop_session};

#[test]
#[serial]
fn start_then_stop_first_instance() {
    start_session("SSL").expect("start session failed");
    qae_mem_init().expect("qae_mem_init failed");
    let inst: Instance = qat::get_first_instance().expect("failed to get first instance");
    inst.start().expect("start instance failed");
    inst.stop().expect("stop instance failed");
    stop_session().expect("stop session failed");
    qae_mem_destroy();
}
