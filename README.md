# pvm-test-harness
Test Harness for running tests on multiple PVMs

# Implementation status

- [ ] Memory loading for PolkaVM
- [ ] Memory comparison
- [ ] JSON-RPC runner

# Submodules

After fetching `ananas` submodule it is required to run:
```
npm ci
npm run build
```
before executing it.

# Running

```
$ cargo run -- --help

Run test harness for PVMs


Usage: pvm-test-harness [OPTIONS] <COMMAND>

Commands:
  json  Execute a JSON test case
  fuzz  Run fuzz testing
  help  Print this message or the help of the given subcommand(s)

Options:
  -c, --config <CONFIG>  toml config file
      --pvm <PVM>        PVMs to run. Can be either 'polkavm', 'stdin=<path>' or jsonrpc=<endpoint>.
  -h, --help             Print help
  -V, --version          Print version
```

To execute a JSON test case, make sure to have the `jamtestvectors` checked out in
a sibling directory.
```
git clone https://github.com/FluffyLabs/jamtestvectors.git
```

After that you can run either a single or multiple JSON files on selected PVMs:
```
cargo run -- --pvm polkavm --pvm stdin=./ananas/bin/stdin.sh json ../jamtestvectors/pvm/programs/inst_add_32.json
```

### Config file

To avoid passing CLI flags for PVM configuration each time one can load a config
file. In [config.toml](./config.toml) file you can find a sample config with two
PVMs configured.

```
cargo run -- -c config.toml json ../jamtestvectors/pvm/programs/inst_add_*.json
```

### Troubleshooting

If you run into any issues make sure to execute with some logs by setting `RUST_LOG`
environment variable.

```
RUST_LOG=debug cargo run -- --pvm polkavm json ../jamtestvectors/pvm/programs/inst_*.json
```

