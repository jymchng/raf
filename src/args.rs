use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(author, version, about, name = "raf")]
pub struct Opts {
    /// Two options: `folder` or `file` to specify whether raf should redact files in a folder or a single file
    /// Example: `raf folder ./tests/test_files -t phone emails -r` means to redact files in `./tests/test_files` with regex types (`-t`) `phone` and `emails` recursively (`-r`)
    /// Example: `raf file ./tests/test_files/docx_1.docx`
    #[clap(subcommand)]
    pub cmd: FileOrFolder,
}

#[derive(Debug, Subcommand)]
pub enum FileOrFolder {
    /// Redact all files in a folder. Optional `-r` flag to indicate if to do so recursively.
    /// Example: `raf folder ./tests/test_files -t phone emails -r` means to redact files in `./tests/test_files` with regex types (`-t`) `phone` and `emails` recursively (`-r`)
    #[clap(name = "folder")]
    Folder(FolderOpts),
    /// Redact a single file with a specified file path.
    /// Example: `raf file ./tests/test_files/docx_1.docx`
    #[clap(name = "file")]
    File(FileOpts),
}

#[derive(Args, Debug)]
pub struct FolderOpts {
    /// `path` of the directory in which all files in it should be redacted.
    /// Example: On Windows => `./tests/test_files`.
    #[clap(parse(from_os_str), required = true)]
    pub path: std::path::PathBuf,

    /// The type of redaction to be applied to the files.
    /// Example: `-t phone emails`, specifies to redact all text that matches the regexes of `phone` and `emails`.
    #[clap(short, long, required = true, multiple_values = true)]
    pub types: Vec<String>,

    /// Whether raf should redact subdirectories recursively. Defaults to `false`, which means raf will only redact files found in the directory specified.
    /// Example: `-t emails -r`, `-r` specifies to redact all sub-directories recursively.
    #[clap(short, long, takes_value = false)]
    pub recursive: bool,
}

#[derive(Args, Debug)]
pub struct FileOpts {
    /// `path` of the directory in which all files should be redacted.
    /// Example, on Windows: `./tests/test_files`.
    #[clap(parse(from_os_str), required = true)]
    pub path: std::path::PathBuf,

    /// The type of redaction to be applied to the files.
    /// Example: `-t phone emails`, specifies to redact all text that matches the regexes of `phone` and `emails`.
    #[clap(short, long, required = true, multiple_values = true)]
    pub types: Vec<String>,
}
