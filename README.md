# Gastronomy UPLC Debugger

Smart contracts on Cardano tend to be very small, so this UPLC Debugger stores the state of the machine at every step of execution; giving the ability to then step forward and backward with ease, and present a nice user interface for doing so.

## Quick Start

```sh
gastronomy run test_data/fibonacci.uplc 03
```

- N - Advance to the next step
- P - Rewind to the previous step
- Q - Quit

## Features

Below you will find the planned and completed features for the Gastronomy debugger:

- [x] Loading and Evaluating UPLC Programs
  - [x] Loading flat encoded UPLC programs
  - [x] Loading UPLC pretty printed programs
  - [x] Passing hex encoded arguments
  - [x] Spent and single-step budgets
  - [ ] Loading transactions with a script context
  - [ ] Better command line arguments
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
- [ ] Graphical interface
  - [ ] Desktop application
  - [ ] Custom renderers for some terms
  - [ ] Better "cause and effect" visualization
  - [ ] Budget heat-map
- [ ] Sourcemap integration
  - [ ] Aiken integration
  - [ ] Plu-ts integration