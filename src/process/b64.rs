use base64::{
    engine::general_purpose::{STANDARD, STANDARD_NO_PAD},
    Engine as _,
};

use crate::{cli::Base64Format, utils::get_reader};

pub fn process_encode(input: &str, format: Base64Format) -> anyhow::Result<String> {
    let mut reader = get_reader(input)?;

    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;

    let encode_data = match format {
        Base64Format::Standard => STANDARD.encode(&buf),
        Base64Format::UrlSafe => STANDARD_NO_PAD.encode(&buf),
    };
    Ok(encode_data)
}

pub fn process_decode(input: &str, format: Base64Format) -> anyhow::Result<Vec<u8>> {
    let mut reader = get_reader(input)?;

    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;

    let buf = buf.trim();

    let decode_data = match format {
        Base64Format::Standard => STANDARD.decode(buf)?,
        Base64Format::UrlSafe => STANDARD_NO_PAD.decode(buf)?,
    };

    Ok(decode_data)
}
