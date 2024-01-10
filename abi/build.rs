use std::process::Command;

fn main() {
    tonic_build::configure()
        .out_dir("src/pb")
        .compile(&["./protos/reservation.proto"], &["protos"])
        .unwrap();

    Command::new("cargo")
        .args(["fmt"])
        .output()
        .expect("Failed to run cargo fmt");

    println!("cargo:rerun-if-changed=protos/reservation.proto");
}
