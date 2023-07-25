use clap::{ArgMatches, Arg, Command};
use reqwest::Client;
use ethers::{
    prelude::{abigen, Abigen},
    providers::{Http, Provider},
    types::Address,
};

use std::env;
use std::fs;
use std::path::Path;

fn usage() {
    println!("{}\n{}\n{}\n{}\n{}", 
             "Usage: ./abi_cli_rs -- ",
             "--cmd <CMD>",
             "--input_path <INPUT>", 
             "--output_type <OUT_TYPE>", 
             "--output_path <OUTPUT_PATH>");
}

fn parse_cli_args() -> clap::ArgMatches {
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

async fn fetch(token: &str, contract_addr: &str) -> Result<String, Box<dyn std::error::Error>> {

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

fn format(cli_args: clap::ArgMatches) -> Result<String, Box<dyn std::error::Error>> {
    // input will be either a raw string or file
    // when the method opens, it will check if the input is a file, aka a path on the current
    // system.
    // if this check fails, it will process as a standard string
    
    // matching - fall structure -> 
    //  solidity file -> if its string, write to file then solc generate abi file to read in
    //                -> if its already a file, then solc generate abi
    //  json          -> if its a string, write to file then load abi
    //                -> if its already a file, then attempt to load abi
    //  ethers        -> WIP
    
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

    // input processing start
    println!("{:?}", Path::new(input).exists());
    if !Path::new(input).exists() {
        println!("creating new file");
        
        // get current directory
        // set up tmp directory
        // write to file in tmp
        // load abi from tmp
        // move to type processing

        // WIP - come back for this
    }

    let contents = fs::read_to_string(input)?;

    /*
    if contents[0..26].eq("// SPDX-License-Identifier") {
        println!("solidity file found");
        // $ solc <contract.sol> --abi
    }*/

    //let test = Abigen::new("MultiCall3", input);
    //println!("{:?}", test);

    //let test1 = abigen!(MultiCall3, input);
    //println!("{:?}", test1);
    
    match (input_type.as_str(), output_type) {
        ("sol", "json") => println!(""),
        ("sol", "json_mini") => println!(""),
        ("sol", "ethers") => println!(""),
        ("json", "json_mini") => println!(""),
        ("json", "ethers") => println!(""),
        ("json_mini", "ethers") => println!(""),
        ("sol", "all") => println!("sol - all"),
        ("json", "all") => println!("json - all"),
        ("json_mini", "all") => println!("json-mini - all"),
        ("ethers", "all") => println!("ethers - all"),
        _ => println!("default"),
    }


    Ok("test".into())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli_args: ArgMatches = parse_cli_args();

    if let Some(cmd) = cli_args.get_one::<String>("command") {
        match cmd.as_str() {
            "format" => { format(cli_args); },
            "fetch" => { fetch("", "").await; },
            _ => { usage(); },
        }
    } else {
        usage();
    }

    Ok(())
}
