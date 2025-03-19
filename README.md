# Gastronomy UPLC Debugger

Smart contracts on Cardano tend to be very small, so this UPLC Debugger stores the state of the machine at every step of execution; giving the ability to then step forward and backward with ease, and present a nice user interface for doing so.

## Quick Start

CLI tool:
```sh
gastronomy-cli run test_data/fibonacci.uplc 03
```

- N - Advance to the next step
- P - Rewind to the previous step
- Q - Quit

GUI:
```
gastronomy
```

### Configuration.

The app will read configuration from environment variables, or from a `.gastronomyrc.toml` file in your home directory.
|Setting|Environment variable|Description|
|---|---|---|
|`blockfrost.key`|`BLOCKFROST_KEY`|The API key to use when querying Blockfrost.|

## Features

Below you will find the planned and completed features for the Gastronomy debugger:

- [x] Loading and Evaluating UPLC Programs
  - [x] Loading flat encoded UPLC programs
  - [x] Loading UPLC pretty printed programs
  - [x] Passing hex encoded arguments
  - [x] Spent and single-step budgets
  - [x] Loading transactions with a script context
- [x] Step through debugging
  - [x] Step forward
  - [x] Display current term
  - [x] Display current context
  - [x] Display current environment
  - [x] Display return values
  - [ ] Relabel variables
  - [ ] Speculative execution with a changed environment
  - [ ] Place bookmarks for easy navigation
- [x] Time-travel Debugging
  - [x] Step backwards
  - [ ] Step to where environment variable introduced
  - [ ] Step backwards through context stack
  - [ ] Step forwards through context stack
- [x] Graphical interface
  - [x] Desktop application
  - [ ] Custom renderers for some terms
  - [ ] Better "cause and effect" visualization
  - [ ] Budget heat-map
- [x] Sourcemap integration
  - [x] Aiken integration (via Aiken fork, contact us!)
  - [ ] Plu-ts integration

