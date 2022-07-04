use klu::read::Archive;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;

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
    let archive = Archive::new(reader).unwrap();

    let mut buf = String::new();
    for fileheader in archive.get_headers() {
        if !fileheader.is_dir() {
            buf.clear();
            let mut file = archive.get_file(fileheader.file_path()).unwrap();
            let _ = file.read_to_string(&mut buf).unwrap();
            println!("{}", buf);
        }
    }
}
