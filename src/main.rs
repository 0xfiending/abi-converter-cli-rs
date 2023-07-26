use clap::ArgMatches;
use abi_tool_rs::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli_args: ArgMatches = parse_cli_args();

    if let Some(cmd) = cli_args.get_one::<String>("command") {
        match cmd.as_str() {
            "format" => { format(cli_args).await; },
            "fetch" => { fetch("", "").await; },
            _ => { usage(); },
        }
    } else {
        usage();
    }

    Ok(())
}
