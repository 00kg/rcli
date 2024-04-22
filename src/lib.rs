mod cli;
mod process;
mod utils;

pub use cli::{
    Base64SubCommand, HttpServeOpts, HttpSubCommand, Opts, SubCommand, TextCryptFormat,
    TextSignFormat, TextSubCommand,
};

pub use process::process_csv;
pub use process::process_decode;
pub use process::process_encode;
pub use process::process_genpass;
pub use process::{process_decrypt, process_encrypt};
pub use process::{process_generate_key, process_text_sign, process_text_verify};

pub use process::process_http_serve;

#[allow(async_fn_in_trait)]
pub trait CmdExector {
    async fn execute(self) -> anyhow::Result<()>;
}
