use anyhow::Result;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use time::{Duration, OffsetDateTime};

use crate::{utils::get_reader, ExpObj};

pub fn process_jwt_sign(
    input: &str,
    secret: &str,
    sub: &str,
    adu: &str,
    exp: ExpObj,
) -> anyhow::Result<String> {
    let mut reader = get_reader(input)?;
    let mut buf = String::new();
    // reader.read_to_end(&mut buf)?;
    reader.read_to_string(&mut buf)?;

    let iat = OffsetDateTime::now_utc();

    let exp = match exp.unit {
        crate::TimeUnit::Month => iat + Duration::days(exp.number * 30),
        crate::TimeUnit::Day => iat + Duration::days(exp.number),
        crate::TimeUnit::Minute => iat + Duration::minutes(exp.number),
    };

    let claims = Claims::new(sub.to_string(), adu.to_string(), buf, iat, exp);

    jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| anyhow::anyhow!("{}", e))
}

pub fn process_jwt_verify(input: &str, secret: &str) -> Result<bool> {
    let mut reader = get_reader(input)?;
    let mut buf = String::new();
    // reader.read_to_end(&mut buf)?;
    reader.read_to_string(&mut buf)?;
    // let buf = buf.trim();

    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_aud = false;

    let token_data = jsonwebtoken::decode::<Claims>(
        buf.trim(),
        &DecodingKey::from_secret(secret.trim().as_bytes()),
        &validation,
    )?;
    println!("{:?}", token_data);

    Ok(true)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Claims {
    sub: String,
    aud: Option<HashSet<String>>,
    input: String,
    #[serde(with = "jwt_numeric_date")]
    iat: OffsetDateTime,
    #[serde(with = "jwt_numeric_date")]
    exp: OffsetDateTime,
}

impl Claims {
    fn new(
        sub: String,
        aud: String,
        input: String,
        iat: OffsetDateTime,
        exp: OffsetDateTime,
    ) -> Self {
        // normalize the timestamps by stripping of microseconds
        let iat = iat
            .date()
            .with_hms_milli(iat.hour(), iat.minute(), iat.second(), 0)
            .unwrap()
            .assume_utc();
        let exp = exp
            .date()
            .with_hms_milli(exp.hour(), exp.minute(), exp.second(), 0)
            .unwrap()
            .assume_utc();
        // let mut aud = HashSet::new();
        // aud.insert(aud_item);

        let aud: HashSet<String> = aud
            .split(',')
            .map(|s| s.trim()) // 移除每个元素两端的空白字符
            .map(|s| s.to_string()) // 将 &str 转换为 String
            .collect(); // 收集到 HashSet 中;

        Self {
            sub,
            aud: Some(aud),
            input,
            iat,
            exp,
        }
    }
}

mod jwt_numeric_date {
    use serde::{self, Deserialize, Deserializer, Serializer};
    use time::OffsetDateTime;

    /// Serializes an OffsetDateTime to a Unix timestamp (milliseconds since 1970/1/1T00:00:00T)
    pub fn serialize<S>(date: &OffsetDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let timestamp = date.unix_timestamp();
        serializer.serialize_i64(timestamp)
    }

    /// Attempts to deserialize an i64 and use as a Unix timestamp
    pub fn deserialize<'de, D>(deserializer: D) -> Result<OffsetDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        OffsetDateTime::from_unix_timestamp(i64::deserialize(deserializer)?)
            .map_err(|_| serde::de::Error::custom("invalid Unix timestamp value"))
    }
}
