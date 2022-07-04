fn main() {
    let mut args = std::env::args().skip(1);
    let base_dir: std::path::PathBuf =
        From::from(args.next().expect("You need to provide a directory"));
    let out_file_path: std::path::PathBuf =
        From::from(args.next().unwrap_or_else(|| "out.klu".to_string()));

    let out_file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(out_file_path)
        .unwrap();

    let bytes_written = klu::write::write_archive_to_path(base_dir, out_file).unwrap();
    dbg!(bytes_written);
}
