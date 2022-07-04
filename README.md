# KLU

This is a simple archive format.
It is split in two parts: 
	- The headers
	- The data

## Headers
	Every file / directory has an header with some information about the itself.
	it contains the file path (ending with a "/" for directory), the filesize, and file offset inside the archive)
	the details on how it is stored is still WIP but currently it is like this: 
	```
		file_size: u64,
		file_offset: u64,
		file_path_size: u16,
		file_path: PathBuf (file_path_size bytes long)
	```

## Data 
	The file's data is stored in a sequential way, where every files are concatanated together and we only choose which part of we take to grab archive's files;

# API

Currently the api is VERY basic: creation of archive from a directory and reading of an archive from a Reader + reading of files.
### Reading/Parsing an archive
You should Wrap the reader in a BufReader because it will do a lot of small reads (like 2-8 bytes read)

### Creating an archive
For the first version, the only way to create an archive is through the file system.
You don't have to wrap the writer in a BufWriter as it is wrapped inside the function.

# CLI

The cli allows you to list files, pack and unpack an archive.
To get the cli, do `cargo install klu --features cli` and the use `klu help` to get the list of commands
