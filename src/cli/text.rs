use std::{fmt, path::PathBuf, str::FromStr};

use crate::CmdExector;

use super::{verify_file, verify_path};
use anyhow::{Ok, Result};
use clap::Parser;
use tokio::fs;

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

impl CmdExector for TextSignOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let signed = crate::process_text_sign(&self.input, &self.key, self.format)?;
        println!("\nhash: {}", signed);
        Ok(())
    }
}

impl CmdExector for TextVerifyOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let verified = crate::process_text_verify(&self.input, &self.key, self.format, &self.sig)?;
        println!("\nverify: {}", verified);
        Ok(())
    }
}

impl CmdExector for TextKeyGenerateOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let key = crate::process_generate_key(self.format)?;
        match self.format {
            TextSignFormat::Blake3 => {
                let name = self.output.join("blake3.txt");
                fs::write(name, &key[0]).await?;
            }
            TextSignFormat::Ed25519 => {
                fs::write(self.output.join("ed25519.sk"), &key[0]).await?;
                fs::write(self.output.join("ed25519.pk"), &key[1]).await?;
            }
        }
        Ok(())
    }
}

impl CmdExector for EncryptOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let ciphertext = crate::process_encrypt(&self.input, &self.key, &self.nonce, self.format)?;
        println!("\nciphertext:{}", ciphertext);
        Ok(())
    }
}

impl CmdExector for DecryptOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let data = crate::process_decrypt(&self.input, &self.key, &self.nonce, self.format)?;
        // println!("\ndecrypted:{}", data);
        eprint!("\ndecrypted:");
        println!("{}", data);
        Ok(())
    }
}

impl CmdExector for TextSubCommand {
    async fn execute(self) -> anyhow::Result<()> {
        match self {
            TextSubCommand::Sign(opts) => opts.execute().await,
            TextSubCommand::Verify(opts) => opts.execute().await,
            TextSubCommand::Generate(opts) => opts.execute().await,
            TextSubCommand::Encrypt(opts) => opts.execute().await,
            TextSubCommand::Decrypt(opts) => opts.execute().await,
        }
    }
}
