// READ.RS

use std::io::Read;
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FileHeader {
    is_dir: bool,
    file_size: u64,
    file_path_length: u16,
    file_offset: u64,
    file_path: std::path::PathBuf,
}

impl FileHeader {
    pub fn is_dir(&self) -> bool {
        self.is_dir
    }
    pub fn file_size(&self) -> u64 {
        self.file_size
    }
    pub fn file_offset(&self) -> u64 {
        self.file_offset
    }
    pub fn file_path(&self) -> &std::path::Path {
        self.file_path.as_path()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Archive<F> {
    headersize: u64,
    files_header: Vec<FileHeader>,
    reader: F,
}

#[derive(Debug, thiserror::Error)]
pub enum ReadError {
    #[error("Not enought bytes where given")]
    NotEnoughBytes { missing: u64 },
    #[error("Unexpected IoError: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Not an KLU archive")]
    NotAnArchive,
    #[error("A Path not valid UTF-8")]
    PathNotUtf8,
    #[error("A path is invalid")]
    InvalidPath,
}

#[derive(Debug, thiserror::Error)]
pub enum VirtualFileError {
    #[error("Missing File: no file matched given path")]
    MissingFile { path: std::path::PathBuf },
    #[error("Given path is a directory")]
    IsDirectory { path: std::path::PathBuf },
}

impl<R: Read> Archive<R> {
    pub fn new(mut reader: R) -> Result<Archive<R>, ReadError> {
        let mut buf = [0x00; 4];
        let num = reader.read(&mut buf[..])?;
        if &buf != b"KLU\xFF" || num < 4 {
            return Err(ReadError::NotAnArchive);
        }
        let mut buf = [0x00; 8];
        if reader.read(&mut buf[..])? < 4 {
            return Err(ReadError::NotEnoughBytes { missing: 4 });
        }
        let headers_size = u64::from_be_bytes(buf);
        let mut current = 0u64;

        let mut files_header = Vec::new();

        while current < headers_size {
            let mut file_size = [0x00; 8];
            let mut offset = [0x00; 8];
            let mut file_path_size = [0x00; 2];
            let num = reader.read(&mut file_size[..])?;
            if num < 8 {
                return Err(ReadError::NotEnoughBytes {
                    missing: headers_size - (current + num as u64),
                });
            }
            current += 8;

            let num = reader.read(&mut offset[..])?;
            if num < 8 {
                return Err(ReadError::NotEnoughBytes {
                    missing: headers_size - (current + num as u64),
                });
            }

            current += 8;
            let num = reader.read(&mut file_path_size[..])?;
            if num < 2 {
                return Err(ReadError::NotEnoughBytes {
                    missing: headers_size - (current + num as u64),
                });
            }

            current += 2;
            let fp_size = u16::from_be_bytes(file_path_size);
            let mut str_buf: Vec<u8> = vec![0x00; fp_size.into()];

            let num = reader.read(&mut str_buf[..])?;
            if num < fp_size.into() {
                return Err(ReadError::NotEnoughBytes {
                    missing: headers_size - (current + num as u64),
                });
            }

            current += fp_size as u64;
            let s = std::str::from_utf8(str_buf.as_slice()).map_err(|_| ReadError::PathNotUtf8)?;
            if s.starts_with('/') || s.contains("/../") {
                return Err(ReadError::InvalidPath);
            }
            files_header.push(FileHeader {
                is_dir: s.ends_with('/'),
                file_size: u64::from_be_bytes(file_size),
                file_path: unsafe {
                    // Safety: We already checked if the given vec/slice is UTF-8 with the `s` variable
                    std::path::PathBuf::from(std::ffi::OsString::from(String::from_utf8_unchecked(
                        str_buf,
                    )))
                },
                file_offset: u64::from_be_bytes(offset),
                file_path_length: fp_size,
            });
        }
        Ok(Archive {
            files_header,
            headersize: headers_size,
            reader,
        })
    }
}

impl<R> Archive<R> {
    pub fn find_header<P: AsRef<std::path::Path>>(&self, path: P) -> Option<&FileHeader> {
        let path = path.as_ref();
        self.files_header.iter().find(|&h| h.file_path == path)
    }

    pub fn get_headers(&self) -> &[FileHeader] {
        self.files_header.as_slice()
    }
}

impl<R: std::io::Read + std::io::Seek> Archive<R> {
    /**/
}

impl Archive<std::fs::File> {
    pub fn get_file<P: AsRef<std::path::Path>>(
        &self,
        path: P,
    ) -> Result<impl std::io::Read + '_, VirtualFileError> {
        let path = path.as_ref();
        let header = self
            .find_header(path)
            .ok_or(VirtualFileError::MissingFile {
                path: path.to_path_buf(),
            })?;

        if header.is_dir {
            return Err(VirtualFileError::IsDirectory {
                path: path.to_path_buf(),
            });
        }

        Ok(VirtualFileRef {
            inner: (&self.reader).take(header.file_size),
            offset: header.file_offset,
            size: header.file_size,
            pos: 0,
        })
    }
}
impl Archive<std::io::BufReader<std::fs::File>> {
    pub fn get_file<P: AsRef<std::path::Path>>(
        &self,
        path: P,
    ) -> Result<impl std::io::Read + '_, VirtualFileError> {
        let path = path.as_ref();
        let header = self
            .find_header(path)
            .ok_or_else(|| VirtualFileError::MissingFile {
                path: path.to_path_buf(),
            })?;

        if header.is_dir {
            return Err(VirtualFileError::IsDirectory {
                path: path.to_path_buf(),
            });
        }

        Ok(VirtualFileRef {
            inner: self.reader.get_ref().take(header.file_size),
            offset: header.file_offset,
            size: header.file_size,
            pos: 0,
        })
    }
}

#[derive(Debug)]
struct VirtualFileRef<'file> {
    inner: std::io::Take<&'file std::fs::File>,
    offset: u64,
    pos: u64,
    size: u64,
}

impl<'file> std::io::Read for VirtualFileRef<'file> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        use std::io::{Seek, SeekFrom};
        self.inner
            .get_mut()
            .seek(SeekFrom::Start(self.offset + self.pos.min(self.size)))?;
        let num = self.inner.read(buf)?;
        self.pos += num as u64;
        Ok(num)
    }
}
