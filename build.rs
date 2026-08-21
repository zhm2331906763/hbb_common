use std::{env, fs, path::PathBuf};

fn main() {
    let out_dir = format!("{}/protos", std::env::var("OUT_DIR").unwrap());

    std::fs::create_dir_all(&out_dir).unwrap();

    protobuf_codegen::Codegen::new()
        .pure()
        .out_dir(out_dir)
        .inputs(["protos/rendezvous.proto", "protos/message.proto"])
        .include("protos")
        .customize(protobuf_codegen::Customize::default().tokio_bytes(true))
        .run()
        .expect("Codegen failed.");

    write_windows_custom_build_metadata();
}

fn write_windows_custom_build_metadata() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os != "windows" {
        return;
    }

    let metadata_path = PathBuf::from(env::var("OUT_DIR").unwrap())
        .join("rustdesk_custom_build_metadata.rs");
    println!("cargo:rerun-if-env-changed=RUSTDESK_ENFORCE_CUSTOM_WINDOWS_BUILD");
    if env::var("RUSTDESK_ENFORCE_CUSTOM_WINDOWS_BUILD").as_deref() != Ok("Y") {
        fs::write(
            metadata_path,
            "pub const CUSTOM_BUILD_MARKER: &str = \"\";\n",
        )
        .expect("failed to write default Windows build metadata");
        return;
    }

    // These variables must exist even when their value is intentionally empty.
    // Failing in the dependency build prevents a stock Windows binary from being
    // published when a matrix job lost its custom environment.
    const REQUIRED: &[&str] = &[
        "RENDEZVOUS_SERVER",
        "RELAY_SERVER",
        "API_SERVER",
        "RS_PUB_KEY",
        "FIXED_PASSWORD",
        "APP_NAME",
        "CONN_TYPE",
        "DISABLE_FILE_TRANSFER",
        "PRE_ELEVATE_SERVICE",
        "HIDE_CONNECTION_MANAGER",
        "HIDE_REMOTE_CONNECTION_NOTIFICATION",
        "HIDE_SETUP_SERVER_TIP",
    ];
    for name in REQUIRED {
        if env::var_os(name).is_none() {
            panic!("missing Windows custom compile environment variable: {name}");
        }
    }

    let value = |name: &str| env::var(name).expect("required variable was checked above");
    let marker = format!(
        "rustdesk-custom;app={};conn={};transfer={};pre={};cm={};remote={};tip={}",
        value("APP_NAME"),
        value("CONN_TYPE"),
        value("DISABLE_FILE_TRANSFER"),
        value("PRE_ELEVATE_SERVICE"),
        value("HIDE_CONNECTION_MANAGER"),
        value("HIDE_REMOTE_CONNECTION_NOTIFICATION"),
        value("HIDE_SETUP_SERVER_TIP"),
    );
    let escaped_marker = format!("{marker:?}");
    fs::write(
        metadata_path,
        format!("pub const CUSTOM_BUILD_MARKER: &str = {escaped_marker};\n"),
    )
    .expect("failed to write Windows custom build metadata");

    for name in REQUIRED {
        println!("cargo:rerun-if-env-changed={name}");
    }
}
