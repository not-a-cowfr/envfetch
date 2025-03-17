<h1 align="center">envfetch</h1>
<h5 align="center">Lightweight cross-platform CLI tool for working with environment variables</h5>
<div align="center">
    <a href="https://github.com/ankddev/envfetch/actions/workflows/build.yml"><img src="https://github.com/ankddev/envfetch/actions/workflows/build.yml/badge.svg" alt="Build status"/></a>
    <a href="https://github.com/ankddev/envfetch/actions/workflows/test.yml"><img src="https://github.com/ankddev/envfetch/actions/workflows/test.yml/badge.svg" alt="Test status"/></a>
    <img alt="GitHub commit activity" src="https://img.shields.io/github/commit-activity/w/ankddev/envfetch">
    <a href="https://app.codecov.io/github/ankddev/envfetch"><img src="https://camo.githubusercontent.com/24e6fbb5fab320f1c87a360fcdf93b0901a6fc04fe0cb070a4083346c7946680/68747470733a2f2f636f6465636f762e696f2f67682f616e6b646465762f656e7666657463682f67726170682f62616467652e7376673f746f6b656e3d37325138463858574b51" /></a>
    <a href="https://crates.io/crates/envfetch"><img src="https://img.shields.io/crates/d/envfetch" alt="crates.io downloads"/></a>
    <a href="https://crates.io/crates/envfetch"><img src="https://img.shields.io/crates/v/envfetch" alt="crates.io version"/></a>
    <a href="https://aur.archlinux.org/packages/envfetch"><img src="https://img.shields.io/aur/version/envfetch" alt="AUR version"/></a>
</div>
<div align="center">
    <img src="https://github.com/user-attachments/assets/261ea1fd-438a-40b0-847d-6a460b7a30a9" />
</div>

# Features
- [x] Print list of all environment variables
- [x] Get value of variable by name
    - [x] Show similar variables if given variable not found
- [x] Set variable (temporary and permanent)
- [x] Delete variable (temporary and permanent)
- [x] Load variables from dotenv-style file (temporary and permanent)
- [x] Add string to the end of variable (temporary and permanent)
- [ ] Set and delete multiple variables at once
- [ ] Interactive mode
  - [x] Basic support
- [ ] Export variables
- [ ] Configuration support
# Get started
## Installing

<a href="https://repology.org/project/envfetch/versions">
    <img src="https://repology.org/badge/vertical-allrepos/envfetch.svg" alt="Packaging status">
</a>

### Install script (Windows, PowerShell)
You can install `envfetch` via install script:
```ps
Invoke-RestMethod "https://raw.githubusercontent.com/ankddev/envfetch/main/install.ps1" | Invoke-Expression
```
If you get an error regarding execution policy, please read the error carefully and determine the execution policy that is right for you. You may try re-running the installation script if you have updated the execution policy.

### AUR (Arch Linux)
[envfetch](https://aur.archlinux.org/packages/envfetch) is available as a package in the [AUR](https://aur.archlinux.org). You can install it with an [AUR helper](https://wiki.archlinux.org/title/AUR_helpers) (e.g. `paru`):
```shell
$ paru -S envfetch
```
Thanks to [@adamperkowski](https://github.com/adamperkowski) for maintaining this package!
### Scoop (Windows)
You can install envfetch from Scoop using this command:
```shell
scoop install https://gist.githubusercontent.com/ankddev/f6314b552aa021f676fc999ec697f833/raw/envfetch.json
```
Note, that it uses [manifest, published only as GitHub Gist.](https://gist.github.com/ankddev/f6314b552aa021f676fc999ec697f833)
### From DEB package (Debian-based Linux)
You can download .deb package from [releases](https://github.com/ankddev/envfetch/releases) and thhen execute:
```sh
sudo dpkg -i <deb-package>
```
### Cargo (from crates.io)
You can install envfetch from Cargo (needs Rust installed):
```shell
$ cargo install envfetch
```
### Cargo (from source)
Also, you can install it from source (needs Rust installed):
```shell
$ cargo install --git https://github.com/ankddev/envfetch envfetch
```
### Download binary
Or, get binary from [GitHub Actions (needs GutHub account)](https://github.com/ankddev/envfetch/actions/) or [releases](https://github.com/ankddev/envfetch/releases/)
## Using
To run envfetch, run `envfetch <COMMAND> <ARGS>` in your terminal.
You can run `envfetch help` to see help message or `envfetch --version` to see program's version.

### Command list
### Interactive
> [!NOTE]
> Interactive mode is currently WIP and only list variables is available.
> See for progress in https://github.com/ankddev/envfetch/issues/17. Also it isn't in latest released ersion `v1.4.0`, if you want to test it,
> wait for `v2.0.0` or build `envfetch` from source yourself

Run interactive mode with TUI.

Usage:
`envfetch interactive`

This will start TUI, where you will be able to work with environment variables.
#### Set
Set environment variable and optionally run process.

Usage:
`envfetch set <KEY> <VALUE> [-- <PROCESS>]`, where:
- `KEY` - name of environment variable
- `VALUE` - value of environment variable
- `PROCESS` - name of process which you want to run (optional if --global is used)

Options:
- `--help`/`-h` - show help message
- `--global`/`-g` - set variable permanently in system environment
  - On Windows: stores in registry
  - On Unix: stores in shell config (.bashrc, .zshrc, or config.fish)

For example:
```shell
$ envfetch set MY_VAR "Hello" -- "npm run"  # temporary for process
$ envfetch set MY_VAR "Hello" --global      # permanent system-wide
```
#### Add
Add value to the end of environment variable and optionally run the process

Usage:
`envfetch add <KEY> <VALUE> [-- <PROCESS>]`, where:
- `KEY` - name of environment variable
- `VALUE` - value of environment variable to add
- `PROCESS` - name of process which you want to run (optional if --global is used)

Options:
- `--help`/`-h` - show help message
- `--global`/`-g` - update variable permanently in system environment
  - On Windows: stores in registry
  - On Unix: stores in shell config (.bashrc, .zshrc, or config.fish)

For example:
```shell
$ envfetch add PATH "../hello.exe" -- "cargo run"  # temporary for process
$ envfetch add MY_VAR "Hello" --global             # permanent system-wide
```
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
Delete variable and optionally start process.

Usage:
`envfetch delete <KEY> [-- <PROCESS>]`, where:
- `KEY` - name of environment variable
- `PROCESS` - name of command to run (optional if --global is used)

Options:
- `--help`/`-h` - show help message
- `--global`/`-g` - delete variable permanently from system environment

For example:
```shell
$ envfetch delete MY_VAR -- "npm run"  # temporary for process
$ envfetch delete MY_VAR --global      # permanent system-wide
```

#### Load
Load environment variables from dotenv-style file and optionally run process.

Usage:
`envfetch load [PROCESS]`, where:
- `PROCESS` - name of process which you want to run (optional if --global is used)

Options:
- `--help`/`-h` - show help message
- `--file <FILE>`/`-f <FILE>` - relative or absolute path to file to read variables from. Note that it must be in .env format.
By default, program loads variables from `.env` file in current directory.
- `--global`/`-g` - load variables permanently into system environment

For example:
```shell
$ envfetch load -- "npm run"                 # temporary for process
$ envfetch load --global                     # permanent system-wide
$ envfetch load --global --file .env.prod    # permanent from specific file
```

> [!NOTE]
> When using `--global` flag:
> - On Windows, variables are stored in the registry under HKEY_CURRENT_USER\Environment
> - On Unix-like systems, variables are stored in shell configuration files (.bashrc, .zshrc, or config.fish)
> 
> Without `--global` flag, variables are only set for the current process run
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
# See Also
- [codewars-api-rs](https://github.com/ankddev/codewars-api-rs) - Rust library for Codewars API
- [conemu-progressbar-go](https://github.com/ankddev/conemu-progressbar-go) - Progress bar for ConEmu for Go
- [terminal-go](https://github.com/ankddev/terminal-go) - Go library for working with ANSI/VT terminal sequences
- [zapret-discord-youtube](https://github.com/ankddev/zapret-discord-youtube) - Zapret build for Windows for fixing Discord and YouTube in Russia or other services
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
