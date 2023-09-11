# abi-converter-cli-rs

Inspired by: https://twitter.com/msolomon44

## Features

- Read in a solidity file or a contract's ABI in .json.
  - Supported outputs -> json, json-mini, ethers
  - The converted ABI is written to a directory and to console.
- Fetch a contract's ABI from Etherscan.


## Usage

The usage of the utility is dependent on the 'command' input.

There are two commands, fetch and format.

### fetch
For fetch, config and address are the relevant options.

The --address option is used to supply a contract address to fetch from Etherscan. This option is required.

The --config option is used to supply a .yaml config with `etherscan_api_key` seeded.

The --config option is not necessary if an .env is used instead.

fetch recommended usage: `$ ./abi_converter_cli -- --cmd fetch --addr 0xcA11bde05977b3631167028862bE2a173976CA11`

### format
For format, the relevant options are input_path, input_type, output_type. All of these options are required.

Valid Input Types
- sol => .sol
- json/json_mini => .json

Valid Output Types
- json => pretty JSON
- json_mini => JSON-minified
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

### Suggested Usage
- Quickly convert the ABI of contract(s) you are developing for readability and/or portability.
- Quickly cache ABI(s) of contracts/interfaces you are interacting with.
   - Convert again if necessary. 

## Demo

Check the `work` directory for examples.


## License

[MIT](https://choosealicense.com/licenses/mit/)
