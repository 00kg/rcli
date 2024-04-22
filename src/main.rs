use std::fs;

use clap::Parser;
use rcli::{
    process_csv, process_decode, process_decrypt, process_encode, process_encrypt,
    process_generate_key, process_genpass, process_text_sign, process_text_verify, Opts,
    SubCommand,
};
use zxcvbn::zxcvbn;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let opts = Opts::parse();

    match opts.cmd {
        SubCommand::Csv(opts) => {
            let output = if let Some(output) = opts.output {
                output.clone()
            } else {
                format!("output.{}", opts.format)
            };
            process_csv(&opts.input, output, opts.format)?;
        }

        SubCommand::GenPass(opts) => {
            let password = process_genpass(
                opts.length,
                opts.no_uppercase,
                opts.no_lowercase,
                opts.no_number,
                opts.no_symbol,
            )?;
            let estimate = zxcvbn(&password, &[])?;
            eprintln!("\nPassword strength: {}", estimate.score());
            print!("{}", password);
        }

        SubCommand::Base64(subcmd) => match subcmd {
            rcli::Base64SubCommand::Encode(opts) => {
                let encode_data = process_encode(&opts.input, opts.format)?;
                println!("{}", encode_data);
            }
            rcli::Base64SubCommand::Decode(opts) => {
                let decode_data = process_decode(&opts.input, opts.format)?;
                // TODO: decoded data might not be string
                let decode_data = String::from_utf8(decode_data)?;
                println!("{}", decode_data);
            }
        },

        SubCommand::Text(subcmd) => match subcmd {
            rcli::TextSubCommand::Sign(opts) => {
                // match opts.format {
                //     rcli::TextSignFormat::Blake3 => process_text_sign(&opts.input, &opts.key, opts.format)?,
                //     rcli::TextSignFormat::Ed25519 => todo!(),
                // }
                let signed = process_text_sign(&opts.input, &opts.key, opts.format)?;
                println!("\nhash: {}", signed);
            }
            rcli::TextSubCommand::Verify(opts) => {
                let verified = process_text_verify(&opts.input, &opts.key, opts.format, &opts.sig)?;
                println!("\nverify: {}", verified);
            }

            rcli::TextSubCommand::Generate(opts) => {
                let key = process_generate_key(opts.format)?;
                match opts.format {
                    rcli::TextSignFormat::Blake3 => {
                        let name = opts.output.join("blake3.txt");
                        fs::write(name, &key[0])?;
                    }
                    rcli::TextSignFormat::Ed25519 => {
                        fs::write(opts.output.join("ed25519.sk"), &key[0])?;
                        fs::write(opts.output.join("ed25519.pk"), &key[1])?;
                    }
                }
            }
            rcli::TextSubCommand::Encrypt(opts) => {
                let ciphertext = process_encrypt(&opts.input, &opts.key, &opts.nonce, opts.format)?;
                println!("\nciphertext:{}", ciphertext);
            }
            rcli::TextSubCommand::Decrypt(opts) => {
                let data = process_decrypt(&opts.input, &opts.key, &opts.nonce, opts.format)?;
                // println!("\ndecrypted:{}", data);
                eprint!("\ndecrypted:");
                println!("{}", data);
            }
        },
        SubCommand::Http(subcmd) => match subcmd {
            rcli::HttpSubCommand::Serve(opts) => {
                rcli::process_http_serve(opts.path, opts.port).await?;
                // println!("Serving at http://0.0.0.0:{}",opts.port);
            }
        },
    }

    Ok(())
}
