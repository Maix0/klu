extern crate clap;
extern crate klu;

use std::io::BufReader;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(name = "klu")]
#[clap(author = "Maix <maix522@gmail.com>")]
#[clap(version = "0.1")]
struct App {
    #[clap(long)]
    /// To print debug info when there is an error
    debug: bool,
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Create an archive from a path
    Pack {
        /// Path to the directory/file to pack
        path: std::path::PathBuf,
        /// Path to the resulting archive (will overwrite if existing)
        out: std::path::PathBuf,
    },
    /// Unpack an archive
    Unpack {
        /// Path to the archive to unpack
        path: std::path::PathBuf,
        /// Where to unpack
        out: std::path::PathBuf,
    },
    /// List files inside an archive
    List {
        /// Path to the archive
        path: std::path::PathBuf,
    },
}

fn main() {
    let app = App::parse();
    // Makeshift tryblock
    let res = (move || {
        match app.command {
            Commands::Pack { path, out } => {
                let file = std::fs::OpenOptions::new()
                    .truncate(true)
                    .read(false)
                    .write(true)
                    .create(true)
                    .open(out)?;
                klu::write::write_archive_to_path(path, file)?;
            }
            Commands::List { path } => {
                let archive_file = std::fs::File::open(path)?;
                let archive = klu::read::Archive::new(BufReader::new(archive_file))?;

                archive
                    .get_headers()
                    .iter()
                    .for_each(|h| println!("{}", h.file_path().display()));
            }
            Commands::Unpack { path, out } => {
                std::fs::create_dir_all(&out)?;
                std::env::set_current_dir(out)?;
                let archive_file = std::fs::File::open(path)?;
                let archive = klu::read::Archive::new(BufReader::new(archive_file))?;

                let headers = archive.get_headers().iter().collect::<Vec<_>>();
                for header in headers {
                    if header.is_dir() {
                        std::fs::create_dir_all(header.file_path())?;
                    } else {
                        let mut file = std::fs::OpenOptions::new()
                            .read(false)
                            .truncate(true)
                            .write(true)
                            .create(true)
                            .open(header.file_path())?;

                        std::io::copy(&mut archive.get_file(header.file_path())?, &mut file)?;
                    }
                }
            }
        };
        Result::<(), Box<dyn std::error::Error>>::Ok(())
    })();

    match res {
        Ok(_) => {}
        Err(e) => eprintln!("An error has occured: {e}\nPass --debug for a debug print"),
    }
}
