mod base64;
mod csv;
mod genpass;
mod text;
use std::path::Path;
use std::path::PathBuf;

pub use self::base64::Base64Format;
pub use self::base64::Base64SubCommand;
pub use self::csv::OutputFormat;
pub use self::text::TextCryptFormat;
pub use self::text::TextSignFormat;
pub use self::text::TextSubCommand;

use self::{csv::CsvOpts, genpass::GenPassOpts};
use clap::Parser;

#[derive(Debug, Parser)]
#[command(name="rcli",version,author,about,long_about=None)]
pub struct Opts {
    #[command(subcommand)]
    pub cmd: SubCommand,
}

#[derive(Debug, Parser)]
pub enum SubCommand {
    #[command(name = "csv", about = "Show CSV, or convert CSV to other formats")]
    Csv(CsvOpts),

    #[command(name = "genpass", about = "Generate a random password")]
    GenPass(GenPassOpts),

    #[command(subcommand)]
    Base64(Base64SubCommand),

    #[command(subcommand)]
    Text(TextSubCommand),
}

fn verify_file(file_name: &str) -> Result<String, &'static str> {
    if file_name == "-" || Path::new(file_name).exists() {
        Ok(file_name.into())
    } else {
        Err("File does not exist")
    }
}

fn verify_path(path: &str) -> Result<PathBuf, &'static str> {
    let p = Path::new(path);
    if p.exists() && p.is_dir() {
        Ok(p.into())
    } else {
        Err("Path does not exist or is not a directory")
    }
}

#[cfg(test)] // cfg()编译宏
mod test {
    use super::*;

    #[test]
    fn test_verify_input_file() {
        assert_eq!(verify_file("-"), Ok("-".into()));
        assert_eq!(verify_file("*"), Err("File does not exist"));
        assert_eq!(verify_file("Cargo.toml"), Ok("Cargo.toml".into()));
    }
}
