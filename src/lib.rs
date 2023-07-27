use clap::{Arg, Command};
use reqwest::Client;
use ethers::{
    prelude::{abigen, Abigen},
    providers::{Http, Provider},
    types::Address,
    abi::Contract,
};

use std::env;
use std::fs;
use std::path::Path;
use std::ffi::OsStr;
use std::process::Stdio;
use std::fs::File;
use std::io::{BufWriter, Write};

use tokio::process::Command as TokioCommand;
use tokio::io::{BufReader, AsyncBufReadExt};
use chrono::{DateTime, Utc};

pub fn usage() {
    println!("{}\n{}\n{}\n{}\n{}",
             "Usage: ./abi_cli_rs -- ",
             "--cmd <CMD>",
             "--input_path <INPUT>",
             "--output_type <OUT_TYPE>",
             "--output_path <OUTPUT_PATH>");
}

pub fn parse_cli_args() -> clap::ArgMatches {
    let cli_args = Command::new("abi_cli_rs")
        .args(&[
            Arg::new("command")
                .long("cmd")
                .short('c')
                .num_args(1)
                .required(true),
            Arg::new("input_path")
                .long("in")
                .short('i')
                .num_args(1),
            Arg::new("input_type")
                .long("itype")
                .short('d')
                .num_args(1),
            Arg::new("output_type")
                 .long("otype")
                 .short('t')
                 .ignore_case(true)
                 .num_args(1),
            Arg::new("output_path")
                 .long("out")
                 .short('o')
                 .num_args(1),
        ])
        .get_matches();

    cli_args
}

pub async fn fetch(token: &str, contract_addr: &str) -> Result<String, Box<dyn std::error::Error>> {

    let url = [
        "https://api.etherscan.io/api?module=contract&action=getabi&address=",
        contract_addr,
        "&apikey=",
        token
    ].concat();

    let response = Client::new()
        .get(url)
        .send()
        .await?
        .text()
        .await?;

    let tmp: serde_json::Value = serde_json::from_str(&response)?;
    //println!("{:?}", tmp["result"]);

    let abi_test = &tmp["result"];

    //println!("{:#}", serde_json::to_writer_pretty(&abi_test).unwrap());

    /*
    abigen!(
        MultiCall3,
        "etherscan:0xcA11bde05977b3631167028862bE2a173976CA11"
    );

    const RPC_URL: &str = "https:://eth.llamarpc.com";
    const CONTRACT_ADDRESS: &str = "0xcA11bde05977b36311670a28862bE2a173976CA11";

    let provider = Provider::<Http>::try_from(RPC_URL)?;
    let client = Arc::new(provider);
    let address: Address = CONTRACT_ADDRESS.parse()?;
    let contract = MultiCall3::new(address, client);
    */

    Ok("".into())
}

pub async fn format(cli_args: clap::ArgMatches) -> Result<String, Box<dyn std::error::Error>> {
    let input = match cli_args.get_one::<String>("input_path") {
        Some(input) => input,
        _ => return Err("format|[input_path] command opt missing|specify a raw string or file path".into()),
    };

    let input_type = match cli_args.get_one::<String>("input_type") {
        Some(input_type) => input_type,
        _ => return Err("format|[input_type] command opt missing|specify json, json_mini, ethers, or sol".into()),
    };

    let output = match cli_args.get_one::<String>("output_path") {
        Some(output) => output.to_owned(),
        _ => String::from(env::current_dir()?.to_str().unwrap()),
    };

    let output_type = match cli_args.get_one::<String>("output_type") {
        Some(output_type) => output_type,
        _ => "all",
    };

    // Not sure if i'll still accept string input, seems funky
    if !Path::new(input).exists() {
        println!("creating new file");

        // get current directory
        // set up tmp directory
        // write to file in tmp
        // load abi from tmp
        // move to type processing

        // WIP - come back for this
    }

    match (input_type.as_str(), output_type) {
        ("sol", "json") => { sol_json_convert(input).await; },
        ("sol", "json_mini") => { sol_json_mini_convert(input).await; },
        ("sol", "ethers") => { sol_ethers_convert(input).await; },
        ("json", "json_mini") => { json_to_mini_convert(input); },
        ("json_mini", "json") => { mini_to_json_convert(input); }
        ("json", "ethers") => { println!(""); },
        ("json_mini", "ethers") => { println!(""); },
        ("sol", "all") => { println!("sol - all"); },
        ("json", "all") => { println!("json - all"); },
        ("json_mini", "all") => { println!("json-mini - all"); },
        ("ethers", "all") => { println!("ethers - all"); },
        _ => { println!("default"); },
    }

    Ok("test".into())
}

/* Checks if file_path parameter is a file on the current system,
 * Checks the first line for the SPDX tag
 * Checks the file for a .sol extension
 * Compiles the file_path using solc --abi
 * Pipes back output from the compilation
 * Produces a minified-JSON ABI
*/
async fn validate_sol(file_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(file_path)?;
    if contents[0..26].eq("// SPDX-License-Identifier") &&
        Path::new(file_path).extension().and_then(OsStr::to_str) == Some("sol")
    {
        let mut output_test = TokioCommand::new("solc")
            .args([file_path, "--abi"])
            .stdout(Stdio::piped())
            .spawn()
            .expect("validate_sol|solc compilation failed");

        let output_contents = output_test
            .stdout
            .take()
            .expect("validate_sol|failed to parse compilation output");
        let mut reader = BufReader::new(output_contents).lines();

        // remove extra lines from output to get raw ABI
        reader.next_line().await?;
        reader.next_line().await?;
        reader.next_line().await?;

        let json_mini = match reader.next_line().await? {
            Some(abi) => return Ok(abi),
            _ => return Err("validate_sol|abi can't be parsed from generated output".into()),
        };
    }

    return Err("validate_sol|the provided input has been deemed not a solidity file".into())
}

/* Creates an ABI from .sol file
 * Converts to pretty JSON
 * Prints to console
 * and writes to file
 * File name format: DD-MM-YY_HH:MM_abi_pretty.json
 */
pub async fn sol_json_convert(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let abi = validate_sol(file_path).await?;
    let tmp: serde_json::Value = serde_json::from_str(&abi)?;
    let pretty_json = serde_json::to_string_pretty(&tmp)?;

    let tmp_dir = create_tmp_directory()?;
    let timestamp = format!("{}", Utc::now().format("%d-%m-%Y_%H:%M"));
    let output_file_path = [&tmp_dir,"/",&timestamp,"_abi_pretty.json"].concat();
    file_write(&output_file_path, pretty_json.clone())?;

    // NOTE - re-format later
    println!("\nCommand: format\nfile: {}\noutput-type: JSON\noutput-file: {}", file_path, output_file_path);
    println!("\n\nPretty JSON Console Output:\n{}", pretty_json);

    Ok(())
}

/* Creates an ABI from .sol file
 * Prints to console
 * and writes to file
 * File name format: DD-MM-YY_HH:MM_abi_mini.json
 */
pub async fn sol_json_mini_convert(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    match validate_sol(file_path).await {
        Ok(abi) => {
            let tmp_dir = create_tmp_directory()?;
            let timestamp = format!("{}", Utc::now().format("%d-%m-%Y_%H:%M"));
            let output_file_path = [&tmp_dir,"/",&timestamp,"_abi_mini.json"].concat();
            file_write(&output_file_path, abi.clone())?;

            println!("\nCommand: format\nfile: {}\noutput-type: JSON-minified\noutput-file: {}", file_path, output_file_path);
            println!("\n\nJSON-minified Console Output:\n{}", abi);
        },
        Err(err) => return Err(err),
    };

    Ok(())
}

// WIP - loading the abi into ethers has been difficult
pub async fn sol_ethers_convert(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let abi = match validate_sol(file_path).await {
        Ok(abi) => abi,
        Err(err) => return Err(err),
    };

    let tmp_dir = create_tmp_directory()?;

    let timestamp = format!("{}", Utc::now().format("%d-%m-%Y_%H:%M"));
    let tmp_file_path = [&tmp_dir,"/",&timestamp,"_abi.json"].concat();
    file_write(&tmp_file_path, abi)?;

    if Path::new(&tmp_file_path).exists() {
        let f = File::open(tmp_file_path)?;
        let test1 = Contract::load(f)?;
        println!("{:?}", test1.constructor());
        println!("{:?}", test1.functions().count());
        println!("{:?}", test1.events().count());
        println!("{:?}", test1.errors().count());
        // fallback - bool
        // receive - bool
    }

    // delete tmp_file here after work is done

    Ok(())
}

pub fn json_to_mini_convert(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(file_path)?;
    let tmp: serde_json::Value = serde_json::from_str(&contents)?;
    let abi = serde_json::to_string(&tmp)?;

    let tmp_dir = create_tmp_directory()?;
    let timestamp = format!("{}", Utc::now().format("%d-%m-%Y_%H:%M"));
    let output_file_path = [&tmp_dir,"/",&timestamp,"_abi_mini.json"].concat();
    file_write(&output_file_path, abi.clone())?;

    println!("\nCommand: format\nfile: {}\noutput-type: JSON-minified\noutput-file: {}", file_path, output_file_path);
    println!("\n\nJSON-minified Console Output:\n{}", abi);

    Ok(())
}

pub fn mini_to_json_convert(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(file_path)?;
    let tmp: serde_json::Value = serde_json::from_str(&contents)?;
    let abi = serde_json::to_string_pretty(&tmp)?;

    let tmp_dir = create_tmp_directory()?;
    let timestamp = format!("{}", Utc::now().format("%d-%m-%Y_%H:%M"));
    let output_file_path = [&tmp_dir,"/",&timestamp,"_abi_pretty.json"].concat();
    file_write(&output_file_path, abi.clone())?;

    println!("\nCommand: format\nfile: {}\noutput-type: JSON\noutput-file: {}", file_path, output_file_path);
    println!("\n\nPretty JSON Console Output:\n{}", abi);

    Ok(())
} 

// Creates a tmp directory based on the current working directory
// Skips, if tmp directory already exists
fn create_tmp_directory() -> Result<String, Box<dyn std::error::Error>> { 
    let current_dir = match env::current_dir()?.to_str() {
        Some(current_dir) => current_dir.to_owned(),
        _ => return Err("sol_ethers_convert|could not create tmp directory|aborting...".into()),
    };
    let tmp_dir = [current_dir,"/tmp".into()].concat();

    if !Path::new(&tmp_dir).exists() {
        fs::create_dir(&tmp_dir)?;
    }

    Ok(String::from(tmp_dir))
}

// Appends contents to the file_path provided
fn file_write(file_path: &str, contents: String) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(file_path.clone())?;
    let mut fd = BufWriter::new(file);
    fd.write_all(contents.as_bytes())?;
    fd.flush()?;

    Ok(())
}
