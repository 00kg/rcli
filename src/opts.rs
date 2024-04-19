use std::path::Path;

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
}

#[derive(Debug, Parser)]
pub struct CsvOpts {
    /// Input file path
    #[arg(short, long, value_parser=verify_input_file)]
    pub input: String,

    /// Output file path
    #[arg(short, long, default_value = "output.json")]
    pub output: String,

    /// Delimiter
    #[arg(short, long, default_value_t = ',')]
    pub delimiter: char,

    /// CSV has header or not
    #[arg(long, default_value_t = true)]
    pub header: bool,
}

fn verify_input_file(file_name: &str) -> Result<String, &'static str> {
    if Path::new(file_name).exists() {
        Ok(file_name.into())
    } else {
        Err("File does not exist")
    }
}
