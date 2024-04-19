mod cli;
mod process;

pub use cli::{Base64SubCommand, Opts, SubCommand};

pub use process::process_csv;
pub use process::process_decode;
pub use process::process_encode;
pub use process::process_genpass;
