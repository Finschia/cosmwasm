use std::env::var;
use std::path::PathBuf;
use tempfile::tempdir;

mod functions;
use functions::{copy_files, find_proto_files, generate_ostracon_lib, get_commitish};

mod constants;
use constants::{
    CUSTOM_FIELD_ATTRIBUTES, CUSTOM_TYPE_ATTRIBUTES, OSTRACON_COMMITISH, OSTRACON_REPO,
};

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let ostracon_lib_target = root
        .join("..")
        .join("..")
        .join("packages")
        .join("ostracon-proto")
        .join("src")
        .join("ostracon.rs");
    let target_dir = root
        .join("..")
        .join("..")
        .join("packages")
        .join("ostracon-proto")
        .join("src")
        .join("prost");
    let out_dir = var("OUT_DIR")
        .map(PathBuf::from)
        .or_else(|_| tempdir().map(|d| d.into_path()))
        .unwrap();
    let ostracon_dir = PathBuf::from(var("OSTRACON_DIR").unwrap_or_else(|_| {
        root.join("..")
            .join("ostracon-go")
            .to_str()
            .unwrap()
            .to_string()
    }));

    println!(
        "[info] => Fetching {} at {} into {:?}",
        OSTRACON_REPO, OSTRACON_COMMITISH, ostracon_dir
    );
    get_commitish(
        &PathBuf::from(&ostracon_dir),
        OSTRACON_REPO,
        OSTRACON_COMMITISH,
    ); // This panics if it fails.

    let proto_paths = vec![ostracon_dir.join("proto")];
    let proto_includes_paths = vec![
        ostracon_dir.join("proto"),
        ostracon_dir.join("third_party").join("proto"),
    ];
    // List available proto files
    let protos = find_proto_files(proto_paths);

    // Compile proto files with added annotations, exchange prost_types to our own
    let mut pb = prost_build::Config::new();
    pb.out_dir(&out_dir);
    for type_attribute in CUSTOM_TYPE_ATTRIBUTES {
        pb.type_attribute(type_attribute.0, type_attribute.1);
    }
    for field_attribute in CUSTOM_FIELD_ATTRIBUTES {
        pb.field_attribute(field_attribute.0, field_attribute.1);
    }
    pb.compile_well_known_types();
    // The below in-place path redirection removes the Duration and Timestamp structs from
    // google.protobuf.rs. We replace them with our own versions that have valid doctest comments.
    // See also https://github.com/danburkert/prost/issues/374 .
    pb.extern_path(
        ".google.protobuf.Duration",
        "super::super::google::protobuf::Duration",
    );
    pb.extern_path(
        ".google.protobuf.Timestamp",
        "super::super::google::protobuf::Timestamp",
    );
    println!("[info] => Creating structs.");
    pb.compile_protos(&protos, &proto_includes_paths).unwrap();

    println!("[info] => Removing old structs and copying new structs.");
    copy_files(&out_dir, &target_dir); // This panics if it fails.
    generate_ostracon_lib(&out_dir, &ostracon_lib_target);

    println!("[info] => Done!");
}
