use std::fs::File;
use std::io::BufReader;

use klu::read::Archive;

fn main() {
    let reader = BufReader::new(
        File::open(format!(
            "{}/{}",
            env!("CARGO_MANIFEST_DIR"),
            std::env::args()
                .nth(1)
                .unwrap_or_else(|| "test-data/single_header.klu".to_string())
        ))
        .unwrap(),
    );
    let archive = Archive::new(reader);

    let _ = dbg!(archive);
}
