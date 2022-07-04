//! WRITE.RS

/*
*
*   EVERYTHING IS BIG-ENDIAN
*
* File structure:
* magic: b"KLU\xFF"
* headerssize: u64,
* Header: {
*   file_size: u64,
*   file_offset: u64,
*   file_path_length: u16
*   file_path: String<file_path_length>,
* } * until headerssize bytes as been read
*/

#[derive(Debug, Clone, PartialEq, Eq)]
struct FilePathNotUTF8(pub std::path::PathBuf);

impl std::fmt::Display for FilePathNotUTF8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} isn't valid utf-8 text", self.0.display())
    }
}

impl std::error::Error for FilePathNotUTF8 {}

struct IncompleteFileHeader {
    is_dir: bool,
    filesize: u64,
    file_path_length: u16,
    file_path: std::path::PathBuf,
}

struct FileHeader {
    is_dir: bool,
    filesize: u64,
    file_offset: u64,
    file_path_length: u16,
    file_path: std::path::PathBuf,
}
pub fn write_archive_to_path<P: AsRef<std::path::Path>, W: std::io::Write>(
    source: P,
    dest: W,
) -> std::io::Result<usize> {
    use std::io::Write;

    let incomplete_headers = create_header_list(source)?;
    let header_size = get_headersize(&incomplete_headers);
    let headers = incomplete_to_complete(header_size, incomplete_headers);
    let mut writer = std::io::BufWriter::new(dest);

    let mut num = 0;
    num += writer.write(b"KLU\xFF")?;
    num += writer.write(&header_size.to_be_bytes()[..])?;
    for header in headers.iter() {
        num += write_header(header, &mut writer)?;
    }
    for header in headers.iter().filter(|h| !h.is_dir) {
        let mut file = std::fs::File::open(&header.file_path)?;
        num += std::io::copy(&mut file, &mut writer)? as usize;
    }
    writer.flush()?;

    Ok(num)
}

fn write_header<W: std::io::Write>(header: &FileHeader, writer: &mut W) -> std::io::Result<usize> {
    let mut num = writer.write(&header.filesize.to_be_bytes())?;
    num += writer.write(&header.file_offset.to_be_bytes())?;
    num += writer.write(&header.file_path_length.to_be_bytes())?;
    num += writer.write(
        header
            .file_path
            .as_os_str()
            .to_str()
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    FilePathNotUTF8(header.file_path.clone()),
                )
            })?
            .as_bytes(),
    )?;
    if header.is_dir {
        num += writer.write(b"/")?;
    }
    Ok(num)
}

fn create_header_list(
    source: impl AsRef<std::path::Path>,
) -> std::io::Result<Vec<IncompleteFileHeader>> {
    let mut headers: Vec<IncompleteFileHeader> = Vec::new();
    for entry in walkdir::WalkDir::new(source).sort_by_file_name() {
        let entry = entry?;
        let metadata = entry.metadata()?;
        let path = entry.into_path();
        let is_dir = metadata.is_dir();
        headers.push(IncompleteFileHeader {
            is_dir,
            filesize: metadata.len(),
            file_path_length: path.as_os_str().len() as u16 + if is_dir { 1 } else { 0 },
            file_path: path,
        });
    }
    Ok(headers)
}

fn get_headersize(h: &[IncompleteFileHeader]) -> u64 {
    h.iter().map(|h| 8 /*file_size*/ + 8 /*file_offset*/ + 2 /*file_path_size*/ + h.file_path_length as u64).sum()
}

fn incomplete_to_complete(
    headersize: u64,
    incomplete: Vec<IncompleteFileHeader>,
) -> Vec<FileHeader> {
    let mut offset = 4 + 8 + headersize;
    incomplete
        .into_iter()
        .map(|h| FileHeader {
            file_path: h.file_path,
            file_path_length: h.file_path_length,
            filesize: if h.is_dir { 0 } else { h.filesize },
            is_dir: h.is_dir,
            file_offset: {
                let old = offset;
                if !h.is_dir {
                    offset += h.filesize;
                }
                old
            },
        })
        .collect()
}
