<h1 align="center">envfetch</h1>
<h5 align="center">Lightweight cross-platform CLI tool for working with environment variables</h5>

# Features
- [x] Print all environment variables
- [x] Get value of variable by name
    - [x] Show similar variables if variable not found
- [x] Set variable
- [x] Delete variable
- [x] Load variables from dotenv-style file
- [ ] Globally set variables
- [ ] Globally delete variables
- [ ] Globally load variables from dotenv-style file
- [ ] Set and delete multiple variables at once
# Get started
## Installing
At this moment you can install envfetch from Cargo:
```shell
cargo install envfetch
```
Also, you can install it from source
```shell
cargo install --git https://github.com/ankddev/envfetch envfetch
```
Or, install from [GitHub Actions](https://github.com/ankddev/envfetch/actions/) or [releases](https://github.com/ankddev/envfetch/releases/)
## Using
To run envfetch, run `envfetch <COMMAND> <ARGS>` in your terminal.
You can run `envfetch help` to see help message or `envfetch --version` to see program's version.
### Command list
#### Set
Set environment variable and run process. Note that variable sets only for one run

Usage:
`envfetch set <KEY> <VALUE> <PROCESS>`, where:
- `KEY` - name of environment variable
- `VALUE` - value of environment variable
- `PROCESS` - name of process which you want to run

Options:
- `--help`/`-h` - show help message

For example:
```shell
envfetch set MY_VAR "Hello" "npm run"
```
It will set environment variable with name `MY_VAR` value "Hello" and start `npm run`

#### Print
Print all environment variables

Usage:
`envfetch print`

Options:
- `--help`/`-h` - show help message

For example:
```shell
$ envfetch print
SHELL = "powershell"
windir = "C:\\Windows"
SystemDrive = "C:"
SystemRoot = "C:\\Windows"
...
```
It will print all environment variables in format `VAR = "VALUE"`.
#### Get
Get value of environment variable

Usage:
`envfetch get <KEY>`, where:
- `KEY` - name of environment variable

Options:
- `--help`/`-h` - show help message
- `--no-similar-names`/`-s` - disable showing similar variables if variable not

For example:
```shell
$ envfetch get MY_VAR
"Hello"
```
It will print value of specified variable.
#### Delete
Delete variable and start process. Note that variable deletes only for one run

Usage:
`envfetch delete <KEY> <PROCESS>`, where:
- `KEY` - name of environment variable
- `PROCESS` - name of command to run

Options:
- `--help`/`-h` - show help message

For example:
```shell
envfetch delete MY_VAR "npm run"
```
It will delete variable `MY_VAR` and run `npm run` command.
#### Load
Load environment variables from dotenv-style file and run process. Note that variables sets only for one run

Usage:
`envfetch load <PROCESS>`, where:
- `PROCESS` - name of process which you want to run

Options:
- `--help`/`-h` - show help message
- `--file <FILE>`/`-f <FILE>` - relative or absolute path to file to read variables from. Note that it must in .env format.
By default, program loads variables from `.env` file in current directory.

For example:
```shell
envfetch load "npm run"
envfetch load "npm run" --file ".env.debug"
```
It will load variables from `.env` or `.env.debug` and start `npm run`
# Building from source
- Install Rust. If it already installed, update with
```shell
$ rustup update
```
- Fork this project using button `Fork` on the top of this page
- Clone your fork (replace `<YOUR_USERNAME>` with your username on GitHub):
```shell
$ git clone https://github.com/<YOUR_USERNAME>/envfetch.git
```
- Go to directory, where you cloned envfetch:
```shell
$ cd envfetch
```
- Run program using Cargo (replace `<COMMAND>` and `<ARGS>` to your command and args):
```shell
$ cargo run -- <COMMAND> <ARGS>
```
# Contributing
- Read [section above to build envfetch from source](#building-from-source)
- Create new branch
- Made your changes
- Test that everything works correctly
- Format and lint code with
```shell
$ cargo fmt
$ cargo clippy --fix
```
- Run tests with
```shell
$ cargo test
```
- Push changes
- Open pull request
