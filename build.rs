use std::{env, fs, path::PathBuf};

fn main() {
    // Re-run if our shim or any headers change.
    println!("cargo:rerun-if-changed=shim/shim.c");

    let src = env::var("QATLIB_SRC").expect("set QATLIB_SRC to qatlib source root");
    let build = env::var("QATLIB_BUILD").unwrap_or_else(|_| format!("{src}/build"));
    let mut incs = vec![
        format!("{build}/include"), // where <qat/...> installed/gens live
        format!("{src}/quickassist/lookaside/access_layer/src/sample_code/performance/framework"),
        format!(
            "{src}/quickassist/lookaside/access_layer/src/sample_code/performance/framework/linux/user_space"
        ),
        format!("{src}/quickassist/lookaside/access_layer/src/sample_code/performance/compression"),
        format!("{src}/quickassist/lookaside/access_layer/src/sample_code/performance/common"),
        format!("{src}/quickassist/lookaside/access_layer/src/sample_code/functional/include"),
        format!("{src}/quickassist/utilities/libusdm_drv"),
        format!("{src}/quickassist/lookaside/access_layer/include"),
        format!("{src}/quickassist/include"),
        format!("{src}/quickassist/include/lac"),
        format!("{src}/quickassist/include/dc"),
        format!("{src}/quickassist/lookaside/access_layer/src/sample_code/busy_loop"),
        format!("{src}/quickassist/lookaside/access_layer/src/common/include"),
        format!("{src}/quickassist/lookaside/access_layer/src/common/compression/include"),
        // OSAL includes are required by many sample headers:
        format!("{src}/quickassist/utilities/osal/include"),
        format!("{src}/quickassist/utilities/osal/src/linux/user_space/include"),
        format!(
            "{src}/quickassist/lookaside/access_layer/src/sample_code/functional/asym/eddsa_sample"
        ),
    ];

    // Keep only existing dirs and warn on missing ones (helps when trees differ)
    incs.retain(|p| {
        let ok = fs::metadata(p).is_ok();
        if !ok {
            println!("cargo:warning=include path not found: {p}");
        }
        ok
    });

    let mut shim = cc::Build::new();
    shim.file("shim/shim.c");
    for i in &incs {
        shim.include(i);
    }
    shim.define("USER_SPACE", None);
    shim.define("_GNU_SOURCE", None);
    shim.flag_if_supported("-std=gnu11");
    shim.flag_if_supported("-Wall");
    shim.flag_if_supported("-Wextra");
    shim.flag_if_supported("-fvisibility=hidden");
    shim.compile("qat_shim_c");

    // If you intend to CALL the sample functions, you must also link them in:
    // Either compile the sample C files (simplest), or link a lib you built.

    // === Option 1: Compile the sample sources directly ===
    // (uncomment and adjust paths if you choose this)
    /*
    let mut sample = cc::Build::new();
    for i in &incs { sample.include(i); }
    sample.define("USER_SPACE", None);
    sample.define("_GNU_SOURCE", None);
    sample.flag_if_supported("-std=gnu11");
    sample.files([
        format!("{src}/quickassist/lookaside/access_layer/src/sample_code/functional/common/cpa_sample_utils.c"),
        format!("{src}/quickassist/lookaside/access_layer/src/sample_code/functional/asym/eddsa_sample/cpa_eddsa_sample.c"),
        format!("{src}/quickassist/lookaside/access_layer/src/sample_code/functional/asym/eddsa_sample/cpa_big_num.c"),
        format!("{src}/quickassist/lookaside/access_layer/src/sample_code/functional/asym/eddsa_sample/cpa_ed_point_operations.c"),
    ]);
    sample.compile("eddsa_sample");
    */

    // === Option 2: Link the library you built from Samples.am ===
    // println!("cargo:rustc-link-search=native={build}/lib"); // wherever libeddsa_sample*.a ended up
    // println!("cargo:rustc-link-lib=static=eddsa_sample_s"); // or the actual name you produced

    // Link QAT statically (names may differ in your build)
    println!("cargo:rustc-link-search=native={build}/lib");
    println!("cargo:rustc-link-lib=static=qat");
    println!("cargo:rustc-link-lib=static=usdm");
    println!("cargo:rustc-link-lib=static=eddsa_sample_s");

    for sys in ["pthread", "dl", "m", "rt", "numa", "crypto"] {
        println!("cargo:rustc-link-lib=dylib={sys}");
    }

    let mut b = bindgen::Builder::default()
        .header("shim/bindings_umbrella.h")
        .layout_tests(false)
        .derive_default(true)
        .clang_arg("-DUSER_SPACE")
        .clang_arg("-D_GNU_SOURCE")
        .clang_arg("--target=x86_64-unknown-linux-gnu");

    for i in &incs {
        b = b.clang_arg("-I").clang_arg(i);
    }

    let b = b
        .allowlist_type("^CpaStatus.*")
        .allowlist_var("^CPA_STATUS.*|^cpa_.*")
        .allowlist_function(
            "^edDsa.*|^qat_eddsa_sign.*|^qat_get_instance.*|^qat_cy.*|^qat_start_session.*|^qat_stop_session.*",
        )
        .allowlist_type("^EdDsa.*");
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    b.generate()
        .expect("bindgen failed")
        .write_to_file(out.join("bindings.rs"))
        .unwrap();
}
