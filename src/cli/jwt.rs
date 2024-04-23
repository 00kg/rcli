use std::str::FromStr;

use anyhow::Result;
use clap::Parser;
use enum_dispatch::enum_dispatch;

use crate::CmdExector;

use super::verify_file;

#[derive(Debug, Parser)]
#[enum_dispatch(CmdExector)]
pub enum JwtSubCommand {
    #[command(about = "jwt sign")]
    Sign(JwtSignOpts),

    #[command(about = "jwt verify")]
    Verify(JwtVerifyOpts),
}

#[derive(Debug, Parser)]
pub struct JwtSignOpts {
    #[arg(short,long,value_parser=verify_file,default_value="-")]
    pub input: String,

    #[arg(long, default_value = "my-secret")]
    pub secret: String,

    #[arg(long, default_value = "acme")]
    pub sub: String,

    /// aud1,aud2
    #[arg(long, default_value = "device1")]
    pub aud: String,

    #[arg(long,value_parser=parse_exp_time,default_value="14D")]
    pub exp: ExpObj,
}

#[derive(Debug, Clone, Copy)]
pub struct ExpObj {
    pub number: i64,
    pub unit: TimeUnit,
}

impl ExpObj {
    fn new(number: i64, unit: TimeUnit) -> Self {
        Self { number, unit }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TimeUnit {
    Month,
    Day,
    Minute,
}

#[derive(Debug, Parser)]
pub struct JwtVerifyOpts {
    #[arg(short,long,value_parser=verify_file,default_value="-")]
    pub input: String,

    #[arg(long, default_value = "my-secret")]
    pub secret: String,
}

fn parse_exp_time(t: &str) -> Result<ExpObj> {
    let time_unit: TimeUnit = t
        .chars()
        .last()
        .unwrap_or(' ')
        .to_string()
        .as_str()
        .parse()?;
    let number_str = &t[..t.len() - 1];
    let number: i64 = number_str.parse().unwrap_or(14);
    Ok(ExpObj::new(number, time_unit))
}

impl FromStr for TimeUnit {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::prelude::v1::Result<Self, Self::Err> {
        match s {
            "M" => Ok(TimeUnit::Month),
            "D" => Ok(TimeUnit::Day),
            "m" => Ok(TimeUnit::Minute),
            _ => Err(anyhow::anyhow!("Invalid format TimeUnit")),
        }
    }
}

impl From<TimeUnit> for &'static str {
    fn from(value: TimeUnit) -> Self {
        match value {
            TimeUnit::Month => "M",
            TimeUnit::Day => "D",
            TimeUnit::Minute => "m",
        }
    }
}

impl CmdExector for JwtSignOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let serialized =
            crate::process_jwt_sign(&self.input, &self.secret, &self.sub, &self.aud, self.exp)?;
        eprint!("\nserialized: ");
        print!("{}", serialized);
        Ok(())
    }
}

impl CmdExector for JwtVerifyOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let verify = crate::process_jwt_verify(&self.input, &self.secret)?;
        println!("verify: {}", verify);
        Ok(())
    }
}
