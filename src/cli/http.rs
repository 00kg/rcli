use std::path::PathBuf;

use anyhow::Ok;
use clap::Parser;
use enum_dispatch::enum_dispatch;

use crate::CmdExector;

use super::verify_path;

#[derive(Debug, Parser)]
#[enum_dispatch(CmdExector)]
pub enum HttpSubCommand {
    #[command(about = "Serve a directory over HTTP")]
    Serve(HttpServeOpts),
}

#[derive(Debug, Parser)]
pub struct HttpServeOpts {
    #[arg(short,long,value_parser=verify_path, default_value=".")]
    pub path: PathBuf,

    #[arg(long, default_value_t = 8080)]
    pub port: u16,
}

impl CmdExector for HttpServeOpts {
    async fn execute(self) -> anyhow::Result<()> {
        crate::process_http_serve(self.path, self.port).await?;
        Ok(())
    }
}
