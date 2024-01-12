use anyhow::Result;

fn main() {
    // show throw error
    read_file("foo.txt").unwrap();
}

fn read_file(path: &str) -> Result<String> {
    let res = std::fs::read_to_string(path)?;

    Ok(res)
}
