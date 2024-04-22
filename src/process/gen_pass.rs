use rand::seq::SliceRandom;

const UPPER: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWER: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
const NUMBER: &[u8] = b"1234567890";
const SYMBOL: &[u8] = b"!@#$%^&*~,.;";

pub fn process_genpass(
    length: u8,
    no_upper: bool,
    no_lower: bool,
    no_number: bool,
    no_symbol: bool,
) -> anyhow::Result<String> {
    let mut rng = rand::thread_rng();
    let mut password = Vec::new();
    let mut chars = Vec::new();

    if !no_upper {
        chars.extend_from_slice(UPPER);
        password.push(
            *UPPER
                .choose(&mut rng)
                .expect("chars won't be empty in this context"),
        );
    }
    if !no_lower {
        chars.extend_from_slice(LOWER);
        password.push(
            *LOWER
                .choose(&mut rng)
                .expect("chars won't be empty in this context"),
        );
    }
    if !no_number {
        chars.extend_from_slice(NUMBER);
        password.push(
            *NUMBER
                .choose(&mut rng)
                .expect("chars won't be empty in this context"),
        );
    }
    if !no_symbol {
        chars.extend_from_slice(SYMBOL);
        password.push(
            *SYMBOL
                .choose(&mut rng)
                .expect("chars won't be empty in this context"),
        );
    }

    for _ in 0..(length - password.len() as u8) {
        // let idx = rng.gen_range(0..chars.len());
        // password.push(chars[idx].into());
        let c = chars
            .choose(&mut rng)
            .expect("chars won't be empty in this context");
        password.push(*c);
    }

    password.shuffle(&mut rng);

    let password = String::from_utf8(password)?;
    // let estimate = zxcvbn(&password, &[])?;

    // print!("{}", password);
    // // output password strength in stderr
    // eprintln!("Password strength: {}", estimate.score());
    Ok(password)
}
