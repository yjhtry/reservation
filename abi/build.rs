use std::process::Command;

trait BuilderExt {
    fn types_attributes(self, paths: &[&str], attributes: &[&str]) -> Self;
    fn fields_attributes(self, path: &str, fields: &[&str], attributes: &[&str]) -> Self;
}

fn main() {
    tonic_build::configure()
        .out_dir("src/pb")
        .types_attributes(
            &["reservation.ReservationQuery"],
            &[
                "#[derive(derive_builder::Builder)]",
                "#[builder(setter(into))]",
            ],
        )
        .fields_attributes(
            "reservation.ReservationQuery",
            &["start", "end"],
            &["#[builder(setter(strip_option))]"],
        )
        .fields_attributes(
            "reservation.ReservationQuery",
            &["page"],
            &[r#"#[builder(default = "1")]"#],
        )
        .fields_attributes(
            "reservation.ReservationQuery",
            &["page_size"],
            &[r#"#[builder(default = "10")]"#],
        )
        .fields_attributes(
            "reservation.ReservationQuery",
            &["is_desc"],
            &[r#"#[builder(default = "false")]"#],
        )
        .fields_attributes(
            "reservation.ReservationQuery",
            &["status"],
            &[r#"#[builder(default = "1")]"#],
        )
        .compile(&["./protos/reservation.proto"], &["protos"])
        .unwrap();

    Command::new("cargo")
        .args(["fmt"])
        .output()
        .expect("Failed to run cargo fmt");

    println!("cargo:rerun-if-changed=protos/reservation.proto");
}

impl BuilderExt for tonic_build::Builder {
    fn types_attributes(self, paths: &[&str], attributes: &[&str]) -> Self {
        paths.iter().fold(self, |acc, path| {
            attributes
                .iter()
                .fold(acc, |acc, attribute| acc.type_attribute(path, attribute))
        })
    }

    fn fields_attributes(self, path: &str, fields: &[&str], attributes: &[&str]) -> Self {
        fields.iter().fold(self, |acc, field| {
            attributes.iter().fold(acc, |acc, attribute| {
                acc.field_attribute(format!("{}.{}", path, field), attribute)
            })
        })
    }
}
