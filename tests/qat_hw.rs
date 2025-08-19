use serial_test::serial;

use qat_shim::qat::{self, Instance, start_session, stop_session};

#[test]
#[serial]
fn start_then_stop_first_instance() {
    start_session("SSL").expect("start session failed");
    let inst: Instance = qat::get_first_instance().expect("failed to get first instance");
    inst.start().expect("start instance failed");
    inst.stop().expect("stop instance failed");
    stop_session().expect("stop session failed");
}
