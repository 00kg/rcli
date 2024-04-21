use std::{fmt, path::PathBuf, str::FromStr};

use super::{verify_file, verify_path};
use anyhow::{Ok, Result};
use clap::Parser;

#[derive(Debug, Parser)]
pub enum TextSubCommand {
    #[command(about = "Sign a message with a private/shared key")]
    Sign(TextSignOpts),
    #[command(about = "Verify a signed message")]
    Verify(TextVerifyOpts),

    #[command(about = "Generate a new key")]
    Generate(TextKeyGenerateOpts),

    #[command(about = "Encrypt Data")]
    Encrypt(EncryptOpts),

    #[command(about = "Decrypt Data")]
    Decrypt(DecryptOpts),
}

#[derive(Debug, Parser)]
pub struct EncryptOpts {
    #[arg(short,long,value_parser=verify_file,default_value="-")]
    pub input: String,

    #[arg(short, long)]
    pub key: String,

    #[arg(short, long, default_value = "000000000000")]
    pub nonce: String,

    #[arg(long, default_value="chacha20poly1305", value_parser=parse_crypt_format)]
    pub format: TextCryptFormat,
}

#[derive(Debug, Parser)]
pub struct DecryptOpts {
    #[arg(short,long,value_parser=verify_file,default_value="-")]
    pub input: String,

    #[arg(short, long)]
    pub key: String,

    #[arg(short, long, default_value = "000000000000")]
    pub nonce: String,

    #[arg(long, default_value="chacha20poly1305", value_parser=parse_crypt_format)]
    pub format: TextCryptFormat,
}

#[derive(Debug, Parser)]
pub struct TextSignOpts {
    #[arg(short,long,value_parser=verify_file,default_value="-")]
    pub input: String,

    #[arg(short,long,value_parser=verify_file)]
    pub key: String,

    #[arg(long, default_value="blake3", value_parser=parse_verify_format)]
    pub format: TextSignFormat,
}

#[derive(Debug, Parser)]
pub struct TextVerifyOpts {
    #[arg(short,long,value_parser=verify_file,default_value="-")]
    pub input: String,

    #[arg(short,long,value_parser=verify_file)]
    pub key: String,

    #[arg(long, default_value="blake3", value_parser=parse_verify_format)]
    pub format: TextSignFormat,

    #[arg(short, long)]
    pub sig: String,
}

#[derive(Debug, Parser)]
pub struct TextKeyGenerateOpts {
    #[arg(long, default_value="blake3", value_parser=parse_verify_format)]
    pub format: TextSignFormat,

    #[arg(short,long,value_parser=verify_path)]
    pub output: PathBuf,
}

#[derive(Debug, Clone, Copy)]
pub enum TextSignFormat {
    Blake3,
    Ed25519,
}

#[derive(Debug, Clone, Copy)]
pub enum TextCryptFormat {
    Chacha20poly1305,
}

fn parse_crypt_format(format: &str) -> Result<TextCryptFormat> {
    format.parse()
}

fn parse_verify_format(format: &str) -> Result<TextSignFormat> {
    format.parse()
}

impl FromStr for TextCryptFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::prelude::v1::Result<Self, Self::Err> {
        match s {
            "chacha20poly1305" => Ok(TextCryptFormat::Chacha20poly1305),
            _ => Err(anyhow::anyhow!("Invalid format")),
        }
    }
}

impl FromStr for TextSignFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::prelude::v1::Result<Self, Self::Err> {
        match s {
            "blake3" => Ok(TextSignFormat::Blake3),
            "ed25519" => Ok(TextSignFormat::Ed25519),
            _ => Err(anyhow::anyhow!("Invalid format")),
        }
    }
}

impl From<TextCryptFormat> for &'static str {
    fn from(value: TextCryptFormat) -> Self {
        match value {
            TextCryptFormat::Chacha20poly1305 => "chacha20poly1305",
        }
    }
}

impl From<TextSignFormat> for &'static str {
    fn from(value: TextSignFormat) -> Self {
        match value {
            TextSignFormat::Blake3 => "blake3",
            TextSignFormat::Ed25519 => "ed25519",
        }
    }
}

impl fmt::Display for TextCryptFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Into::<TextCryptFormat>::into(*self))
    }
}

impl fmt::Display for TextSignFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Into::<TextSignFormat>::into(*self))
    }
}
