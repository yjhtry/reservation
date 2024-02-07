use std::process::Command;

trait BuilderExt {
    fn types_attributes(self, paths: &[&str], attributes: &[&str]) -> Self;
    fn fields_attributes(self, path: &[&str], fields: &[&str], attributes: &[&str]) -> Self;
}

fn main() {
    tonic_build::configure()
        .out_dir("src/pb")
        .types_attributes(
            &[
                "reservation.ReservationQuery",
                "reservation.ReservationFilter",
            ],
            &[
                "#[derive(derive_builder::Builder)]",
                "#[builder(setter(into), default)]",
            ],
        )
        .fields_attributes(
            &["reservation.ReservationQuery"],
            &["start", "end"],
            &["#[builder(setter(strip_option))]"],
        )
        .fields_attributes(
            &["reservation.ReservationQuery"],
            &["page"],
            &[r#"#[builder(default = "1")]"#],
        )
        .fields_attributes(
            &[
                "reservation.ReservationQuery",
                "reservation.ReservationFilter",
            ],
            &["page_size"],
            &[r#"#[builder(default = "10")]"#],
        )
        .fields_attributes(
            &[
                "reservation.ReservationQuery",
                "reservation.ReservationFilter",
            ],
            &["is_desc"],
            &[r#"#[builder(default = "false")]"#],
        )
        .fields_attributes(
            &[
                "reservation.ReservationQuery",
                "reservation.ReservationFilter",
            ],
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

    fn fields_attributes(self, path: &[&str], fields: &[&str], attributes: &[&str]) -> Self {
        path.iter().fold(self, |acc, path| {
            fields.iter().fold(acc, |acc, field| {
                attributes.iter().fold(acc, |acc, attribute| {
                    acc.field_attribute(format!("{}.{}", path, field), attribute)
                })
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_types_attributes() {
        let builder = tonic_build::configure().types_attributes(
            &["reservation.ReservationQuery"],
            &[
                "#[derive(derive_builder::Builder)]",
                "#[builder(setter(into), default)]",
            ],
        );

        assert_eq!(
            builder,
            tonic_build::configure()
                .type_attribute(
                    "reservation.ReservationQuery",
                    "#[derive(derive_builder::Builder)]"
                )
                .type_attribute(
                    "reservation.ReservationQuery",
                    "#[builder(setter(into), default)]"
                )
        );
    }

    #[test]
    fn test_fields_attributes() {
        let builder = tonic_build::configure().fields_attributes(
            "reservation.ReservationQuery",
            &["start", "end"],
            &["#[builder(setter(strip_option))]"],
        );

        assert_eq!(
            builder,
            tonic_build::configure()
                .field_attribute(
                    "reservation.ReservationQuery.start",
                    "#[builder(setter(strip_option))]"
                )
                .field_attribute(
                    "reservation.ReservationQuery.end",
                    "#[builder(setter(strip_option))]"
                )
        );
    }
}
