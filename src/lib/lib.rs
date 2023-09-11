use chrono::Utc;
use clap::{Arg, Command};
use reqwest::Client;
use ethers::abi::Contract;
use std::{
    env,
    fs,
    fs::File,
    io::{BufWriter, Write, Read},
    process::Stdio,
    ffi::OsStr,
    path::Path,
    collections::BTreeMap,
};
use tokio::{
    process::Command as TokioCommand,
    io::{BufReader, AsyncBufReadExt},
};

pub fn usage() {
    println!("{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
             "Usage: ./abi_cli_rs -- ",
             "--cmd <CMD>",
             "--in <INPUT_PATH>",
             "--itype <INPUT_TYPE>",
             "--otype <OUTPUT_TYPE>",
             "--out <OUTPUT_PATH>",
             "--addr <CONTRACT_ADDR>",
             "--conf <CONFIG>",);
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
            Arg::new("config")
                .long("conf")
                .short('f')
                .num_args(1),
            Arg::new("address")
                .long("addr")
                .short('a')
                .num_args(1),
        ])
        .get_matches();

    cli_args
}

/* Fetchs the contract ABI for the provided contract_addr
 * ETHERSCAN_API_KEY can be set as an environment variable
 * or can be set by a yaml configuration file in the src directory
 */
pub async fn fetch(cli_args: clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {  
    // Parse CLI args
    let conf = match cli_args.get_one::<String>("config") {
        Some(config) => config,
        _ => "",  // checks for .env below
    };

    let contract_addr = match cli_args.get_one::<String>("address") {
        Some(addr) => addr,
        _ => return Err("fetch|[address] opt not found".into()),
    };

    let output_path = match cli_args.get_one::<String>("output_path") {
        Some(out) => out,
        _ => "",     // writes to /tmp
    };

    let token = get_token(conf)?;

    let url = [
        "https://api.etherscan.io/api?module=contract&action=getabi&address=",
        contract_addr,
        "&apikey=",
        &token
    ].concat();

    let response = Client::new()
        .get(url)
        .send()
        .await?
        .text()
        .await?;

    let tmp: serde_json::Value = serde_json::from_str(&response)?;
    
    let abi_format: serde_json::Value = match &tmp["result"].as_str() {
        Some(abi) => serde_json::from_str(abi)?,
        None => return Err("".into()),
    };
    let json = serde_json::to_string_pretty(&abi_format)?;

    let file_name = [contract_addr, ".json"].concat();
    let output_file_path: String = format_output_path(output_path, &file_name)?;
    file_write(&output_file_path, json.clone())?;

    println!("{:-<1$}", "", 75);
    println!("Command: fetch\ncontract-address: {}\noutput-type: JSON\noutput-file: {}", contract_addr, output_file_path);
    println!("\n\nPretty JSON Console Output:\n{}", json);

    Ok(())
}

pub async fn format(cli_args: clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    // parse CLI args
    let input = match cli_args.get_one::<String>("input_path") {
        Some(input) => input,
        _ => return Err("format|[input_path] command opt missing|specify a raw string or file path".into()),
    };

    let input_type = match cli_args.get_one::<String>("input_type") {
        Some(input_type) => input_type,
        _ => return Err("format|[input_type] command opt missing|specify json, json_mini, ethers, or sol".into()),
    };

    let output_type = match cli_args.get_one::<String>("output_type") {
        Some(output_type) => output_type,
        _ => "all",
    };

    let output = match cli_args.get_one::<String>("output_path") {
        Some(out) => out,
        _ => "",
    };

    // Convert, input_type -> output_type
    // valid input_types: sol, json, json_mini
    // valid output_types: json, json_mini, ethers, all
    match (input_type.as_str(), output_type) {
        ("sol", "json") => { sol_json_convert(input, output).await?; },
        ("sol", "json_mini") => { sol_json_mini_convert(input, output).await?; },
        ("sol", "ethers") => { sol_ethers_convert(input, output).await?; },
        ("json", "json_mini") => { json_to_mini_convert(input, output)?; },
        ("json", "ethers") => { json_ethers_convert(input, output)?; },
        ("json_mini", "json") => { mini_to_json_convert(input, output)?; }
        ("json_mini", "ethers") => { json_ethers_convert(input, output)?; },                                          
        ("sol", "all") => { 
            sol_json_convert(input, output).await?;
            sol_json_mini_convert(input, output).await?;
            sol_ethers_convert(input, output).await?;
        },
        ("json", "all") => { 
            json_to_mini_convert(input, output)?;
            json_ethers_convert(input, output)?;
        },
        ("json_mini", "all") => { 
            mini_to_json_convert(input, output)?;
            json_ethers_convert(input, output)?;
        },
        _ => { },  // no-op
    }

    Ok(())
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

    // Checks for License ID + .sol extension
    if contents[0..26].eq("// SPDX-License-Identifier") &&
        Path::new(file_path).extension().and_then(OsStr::to_str) == Some("sol")
    {
        // Compile the file + generate ABI
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

        match reader.next_line().await? {
            Some(abi) => return Ok(abi),
            _ => return Err("validate_sol|ABI can't be parsed from generated output".into()),
        };
    }

    return Err("validate_sol|the provided path is not a solidity file".into())
}

/* Creates an ABI from .sol file
 * Converts to pretty JSON
 * Prints to console
 * and writes to file
 * File name format: DD-MM-YY_HH:MM_abi_pretty.json
 */
async fn sol_json_convert(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let abi = validate_sol(input_path).await?;
    let tmp: serde_json::Value = serde_json::from_str(&abi)?;
    let pretty_json = serde_json::to_string_pretty(&tmp)?;
    
    let output_file_path = format_output_path(output_path, "abi_pretty.json")?;
    file_write(&output_file_path, pretty_json.clone())?;

    print_abi("Pretty JSON", input_path, &output_file_path, &pretty_json);
    Ok(())
}

/* Creates an ABI from .sol file
 * Prints to console
 * and writes to file
 * File name format: DD-MM-YY_HH:MM_abi_mini.json
 */
async fn sol_json_mini_convert(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    match validate_sol(input_path).await {
        Ok(abi) => {
            let output_file_path = format_output_path(output_path, "abi_mini.json")?;
            file_write(&output_file_path, abi.clone())?;

            print_abi("JSON-minified", input_path, &output_file_path, &abi);
        },
        Err(err) => return Err(err),
    };

    Ok(())
}

async fn sol_ethers_convert(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let abi = match validate_sol(input_path).await {
        Ok(abi) => abi,
        Err(err) => return Err(err),
    };

    let tmp_dir = create_tmp_directory()?;
    let timestamp = format!("{}", Utc::now().format("%d-%m-%Y_%H:%M"));
    let tmp_file_path = [&tmp_dir,"/",&timestamp,"_abi.json"].concat();
    file_write(&tmp_file_path, abi)?;

    if Path::new(&tmp_file_path).exists() {
        let f = File::open(&tmp_file_path)?;
        let contract = Contract::load(f)?;

        let contract_func = contract.functions();

        let mut abi = String::from("[\n");

        if contract.functions().count() > 0 {
            contract_func.for_each(|x| {
                abi.push_str(&["  \"", &x.signature(), "\",\n"].concat());
            });
        }

        abi.push_str("]");  // closing bracket
        
        if abi.len() > 5 {
            let output_file_path: String = format_output_path(output_path, "abi_ethers.json")?;
            file_write(&output_file_path, abi.clone())?;

            print_abi("Ethers-rs", input_path, &output_file_path, &abi);
        } else {
            return Err("Contract ABI could not be read and parsed. Try a different input...".into())
        }
    }

    fs::remove_file(tmp_file_path)?;

    Ok(())
}

fn json_to_mini_convert(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(input_path)?;
    let tmp: serde_json::Value = serde_json::from_str(&contents)?;
    let abi = serde_json::to_string(&tmp)?;

    let output_file_path: String = format_output_path(output_path, "abi_mini.json")?;
    file_write(&output_file_path, abi.clone())?;

    print_abi("JSON-minified", input_path, &output_file_path, &abi);
    Ok(())
}

fn mini_to_json_convert(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(input_path)?;
    let tmp: serde_json::Value = serde_json::from_str(&contents)?;
    let abi = serde_json::to_string_pretty(&tmp)?;
 
    let output_file_path: String = format_output_path(output_path, "abi_pretty.json")?;
    file_write(&output_file_path, abi.clone())?;

    print_abi("Pretty JSON", input_path, &output_file_path, &abi);
    Ok(())
} 

fn json_ethers_convert(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> { 
    if Path::new(&input_path).exists() {
        let f = File::open(input_path)?;
        let contract = Contract::load(f)?;

        let contract_func = contract.functions();
        let mut abi = String::from("[\n");     // start

        if contract.functions().count() > 0 {
            contract_func.for_each(|x| {
                abi.push_str(&["  \"", &x.signature(), "\",\n"].concat());
            });
        }

        abi.push_str("]");  // closing bracket
        
        if abi.len() > 8 {
            let output_file_path: String = format_output_path(output_path, "abi_ethers.json")?;
            file_write(&output_file_path, abi.clone())?;

            print_abi("Ethers-rs", input_path, &output_file_path, &abi);
        } else {
            return Err("Contract ABI could not be read and parsed. Try a different input...".into())
        }
    }

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

fn get_token(path: &str) -> Result<String, Box<dyn std::error::Error>> {
   let token: String;
    
    match path.is_empty() {
        true => {
            // look for ETHERSCAN_API_KEY in .env
            token = match env::var("ETHERSCAN_API_KEY") {
                Ok(t) => t,
                Err(err) => {
                    println!("{:?}", err);
                    return Err("get_token|ETHERSCAN_API_KEY not found".into());
                },
            };
        },
        false => {
            // look for .yaml config
            let mut yaml_config = File::open(
                String::from(path.to_owned())
            )
            .expect(&format!("get_token|{} cannot be opened", path));

            let mut file_data = String::new();
            yaml_config
                .read_to_string(&mut file_data)
                .expect(&format!("get_token|yaml_config cannot be read"));

            let conf: BTreeMap<String, String> = 
                serde_yaml::from_str(&file_data)
                .expect("get_token|serde_yaml parse failed. conf creation aborted...".into());

            token = conf.get("etherscan_api_key")
                .expect("get_token|etherscan_api_key not found")
                .clone();
        },
    }

    match token.is_empty() {
        true => return Err("get_token|etherscan_key is empty".into()),
        false => return Ok(token),
    } 
}

fn format_output_path(output_path: &str, file_suffix: &str) -> Result<String, Box<dyn std::error::Error>> {
    let output_file_path: String;
    let timestamp = format!("{}", Utc::now().format("%d-%m-%Y_%H:%M"));
    match output_path.is_empty() {
        true => {
            let tmp_dir = create_tmp_directory()?;
            output_file_path = [&tmp_dir,"/",&timestamp,"_",file_suffix].concat();
        },
        false => { 
            output_file_path = [output_path,"/",&timestamp,"_",file_suffix].concat();
        },
    }
    Ok(output_file_path)
}

fn print_abi(
    output_type: &str,
    input_path: &str, 
    output_path: &str, 
    abi: &str
) {
    println!("{:-<1$}", "", 75);
    println!("Command: format\nfile: {}\noutput-type: {}\noutput-file: {}", input_path, output_type, output_path);
    println!("\n\nConsole Output:\n{}", abi);
}
