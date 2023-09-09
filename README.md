# abi-converter-cli-rs

A simple CLI utility that converts a contract's ABI into different formats, written in pure rust.

Inspired by: https://twitter.com/msolomon44

## Features

- Read in a contract's ABI in .sol or .json formats.
  - Supported outputs -> json, json-mini, ethers
  - The converted ABI is written to a directory and to console.
- Fetch a contract's ABI from Etherscan.


## Usage

The usage of the utility is dependent on the 'command' input.

There are two commands, fetch and format.

For fetch, config and address are the relevant options.

The --address option is used to supply a contract address to fetch from Etherscan. This option is required.

The --config option is used to supply a .yaml config with `etherscan_api_key` seeded.

The --config option is not necessary if an .env is used instead.

fetch recommended usage: `$ ./abi_converter_cli -- --cmd fetch --addr 0xcA11bde05977b3631167028862bE2a173976CA11`

For format, the relevant options are input_path, input_type, output_type. All of these options are required.

### Valid Input Types
- sol => .sol
- json => .json

### Valid Output Types
- json => pretty JSON
- json-mini => JSON-minified
- ethers => ethers-rs
- all => converts to all available formats, i.e. if sol -> all, the output will be json, json-mini, and ethers.

format recommended usage: `$ ./abi_converter_cli -- --cmd format --in tmp.json --itype json --otype ethers` 

```bash
  Usage: abi_converter_cli [OPTIONS] --cmd <command>

  Options:
    -c, --cmd <command>
    -i, --in <input_path>
    -d, --itype <input_type>
    -t, --otype <output_type>
    -f, --conf <config>
    -a, --addr <address>
    -h, --help                 Print help
```


## Demo

Insert gif or link to demo


## License

[MIT](https://choosealicense.com/licenses/mit/)
