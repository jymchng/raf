use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(author, version, about, name = "raf")]
pub struct Opts {
    #[clap(subcommand)]
    pub cmd: FileOrFolder,
}

#[derive(Debug, Subcommand)]
pub enum FileOrFolder {
    #[clap(name = "folder")]
    Folder(FolderOpts),
    #[clap(name = "file")]
    File(FileOpts),
}

#[derive(Args, Debug)]
pub struct FolderOpts {
    /// `path` of the directory in which all files should be redacted, e.g. ./tests/test_files
    #[clap(parse(from_os_str), required = true)]
    pub path: std::path::PathBuf,

    /// The type of redaction to be applied to the files, e.g. -t sgNRIC emails
    #[clap(short, long, required = true, multiple_values = true)]
    pub types: Vec<String>,

    /// The type of redaction to be applied to the files, e.g. -t sgNRIC emails
    #[clap(short, long, required = false)]
    pub recursive: Vec<String>,
}

#[derive(Args, Debug)]
pub struct FileOpts {
    #[clap(parse(from_os_str), required = true)]
    pub path: std::path::PathBuf,
    #[clap(short, long, required = true, multiple_values = true)]
    pub types: Vec<String>,
}
