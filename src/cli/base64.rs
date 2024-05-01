use core::fmt;
use std::str::FromStr;

use crate::CmdExector;

use super::verify_file;
use clap::Parser;
use enum_dispatch::enum_dispatch;

#[derive(Debug, Parser)]
#[enum_dispatch(CmdExector)]
pub enum Base64SubCommand {
    #[command(name = "encode", about = "")]
    Encode(Base64EncodeOpts),
    #[command(name = "decode", about = "")]
    Decode(Base64DecodeOpts),
}
impl CmdExector for Base64EncodeOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let encode_data = crate::process_encode(&self.input, self.format)?;
        println!("{}", encode_data);
        Ok(())
    }
}

impl CmdExector for Base64DecodeOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let decode_data = crate::process_decode(&self.input, self.format)?;
        // TODO: decoded data might not be string
        let decode_data = String::from_utf8(decode_data)?;
        println!("{}", decode_data);
        Ok(())
    }
}

#[derive(Debug, Parser)]
pub struct Base64EncodeOpts {
    #[arg(short,long,value_parser=verify_file,default_value="-")]
    pub input: String,
    #[arg(long,default_value="standard",value_parser=parse_format)]
    pub format: Base64Format,
}

#[derive(Debug, Parser)]
pub struct Base64DecodeOpts {
    #[arg(short,long,value_parser=verify_file,default_value="-")]
    pub input: String,

    #[arg(long,default_value="standard",value_parser=parse_format)]
    pub format: Base64Format,
}

#[derive(Debug, Clone, Copy)]
pub enum Base64Format {
    Standard,
    UrlSafe,
}

fn parse_format(format: &str) -> Result<Base64Format, anyhow::Error> {
    format.parse()
}

impl FromStr for Base64Format {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "standard" => Ok(Base64Format::Standard),
            "urlsafe" => Ok(Base64Format::UrlSafe),
            _ => Err(anyhow::anyhow!("Invalid format")),
        }
    }
}

impl From<Base64Format> for &'static str {
    fn from(value: Base64Format) -> Self {
        match value {
            Base64Format::Standard => "standard",
            Base64Format::UrlSafe => "urlsafe",
        }
    }
}

impl fmt::Display for Base64Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Into::<&str>::into(*self))
    }
}
