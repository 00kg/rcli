use std::fs;

use anyhow::Ok;
use csv::Reader;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::opts::OutputFormat;

#[derive(Debug, Deserialize, Serialize)]
struct Player {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Position")]
    position: String,
    #[serde(rename = "DOB")]
    dob: String,
    #[serde(rename = "Nationality")]
    nationality: String,
    #[serde(rename = "Kit Number")]
    kit: u8,
}

pub fn process_csv(input: &str, output: String, format: OutputFormat) -> anyhow::Result<()> {
    // let mut reader = Reader::from_path(opts.input)?;
    // // let records = reader
    // //     .deserialize()
    // //     .map(|record|record.unwrap())
    // //     .collect::<Vec<Player>>();
    // // println!("{:?}",records);
    // for result in reader.deserialize(){
    //     let record: Player = result?;
    //     println!("{:?}",record);
    // }
    let mut reader = Reader::from_path(input)?;
    let mut ret = Vec::with_capacity(128);
    let headers = reader.headers()?.clone();
    for result in reader.records() {
        let record = result?;

        // headers.iter() -> 使用headers的迭代器
        // record.iter() -> 使用record的迭代器
        // zip 将两个迭代器合并为一个元祖的迭代器(拉链?) -> (header, record),
        // serde_json::value 可以接受 Object(Map<String, Value>),格式数据
        let json_value = headers.iter().zip(record.iter()).collect::<Value>();
        // println!("{:?}",json_value);

        ret.push(json_value)
    }
    let content = match format {
        OutputFormat::Json => serde_json::to_string_pretty(&ret)?,
        OutputFormat::Yaml => serde_yaml::to_string(&ret)?,
        OutputFormat::Toml => todo!(),
    };

    fs::write(output, content)?;

    Ok(())
}
