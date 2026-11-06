<div align="center">

<img width="100%" alt="header" src="https://capsule-render.vercel.app/api?type=waving&height=210&text=Teneo%20Beacon%20Bot&fontAlign=50&fontAlignY=36&fontSize=56&desc=Relay%20Nodes%20%7C%20Heartbeat%20%7C%20Beacon%20Claim%20%7C%20Proxy%20%7C%20Multi-Account&descAlign=50&descAlignY=58"/>

<img alt="typing" src="https://readme-typing-svg.demolab.com?font=Inter&size=18&duration=3000&pause=650&center=true&vCenter=true&width=900&lines=Auto+Node+Keys+%7C+Issued+Once+%7C+Cached+In+nodeId.json;Relay+Channels+%7C+One+Socket+Per+Node+%7C+Auto+Reconnect;Auto+Heartbeat+%7C+Account+Points+Each+Interval;Auto+Beacon+Claim+%7C+Challenge+Token+%7C+Only+When+Due;Proxy+Support+%7C+API+Calls+%7C+Masked+In+Logs;Multi-Account+%7C+Sequential+Processing+Per+Cycle"/>

<p>
  <img alt="rust" src="https://img.shields.io/badge/Rust-2021-f74c00?logo=rust&logoColor=white"/>
  <img alt="platform" src="https://img.shields.io/badge/Platform-Teneo%20Beacon-111111"/>
  <img alt="multi-account" src="https://img.shields.io/badge/Multi--Account-Supported-111111"/>
  <img alt="proxy" src="https://img.shields.io/badge/Proxy-Supported-111111"/>
  <img alt="build" src="https://github.com/Yuurisan-N1/Teneo-Beacon/actions/workflows/build.yml/badge.svg"/>
  <img alt="author" src="https://img.shields.io/badge/by-Yuurisandesu-111111"/>
</p>

<p>
  <b>Teneo Beacon Bot</b> is a node and beacon automation client for the Teneo Beacon gateway, written in Rust.<br/>
  It issues node keys for each account once and caches them, keeps one relay socket open per node, sends the account heartbeat on a fixed interval, and on every cycle reads the beacon state and claims the fragments when the boost is due, all across multiple accounts with proxy support and a live countdown between cycles.<br/>
  Built and distributed by <b>Yuurisandesu</b>.
</p>

</div>

---

## Table of Contents

- [Requirements](#requirements)
- [Install Rust](#install-rust)
- [Installation](#installation)
- [Configuration](#configuration)
- [First-Time Setup](#first-time-setup)
- [Running the Bot](#running-the-bot)
- [Run Modes](#run-modes)
- [Packaging](#packaging)
- [Download Prebuilt Binary](#download-prebuilt-binary)
- [Features](#features)
- [Troubleshooting](#troubleshooting)
- [File Structure](#file-structure)
- [Disclaimer](#disclaimer)

---

## Requirements

- Rust `1.75+` (includes `cargo`, only needed if building from source)
- A C and C++ compiler plus CMake (needed to build the TLS library when compiling from source)
- Git
---

## Install Rust

### Linux / macOS

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

Verify:

```bash
rustc --version
```

Install the build tools:

```bash
# Debian / Ubuntu
sudo apt update && sudo apt install -y build-essential cmake clang pkg-config git screen

# macOS (Homebrew)
xcode-select --install
brew install cmake
```

### Windows

Download and run the installer from https://rustup.rs, then install Visual Studio Build Tools with the C++ workload and CMake. Reopen the terminal and verify:

```cmd
rustc --version
```

### Termux (Android)

```bash
pkg update && pkg install proot-distro
proot-distro install ubuntu
proot-distro login ubuntu
```

Then inside Ubuntu:

```bash
apt update && apt install -y curl git screen build-essential cmake clang pkg-config
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

---

## Installation

**Clone the repository:**

```bash
git clone https://github.com/Yuurisan-N1/Teneo-Beacon.git
cd Teneo-Beacon
```

**Set the build values:**

The gateway addresses and the request identity are compiled into the binary, so five environment variables must be set in the shell before building. A build without them fails at compile time.

| Variable | Purpose |
|---|---|
| `ORIGIN` | Origin header sent with every request |
| `API_KEY` | Value of the `x-api-key` header |
| `GATEWAY` | Base address of the beacon gateway |
| `API` | Base address of the points and heartbeat API |
| `WS` | Address of the relay websocket |

```bash
# Linux / macOS / Termux
export ORIGIN="<value>"
export API_KEY="<value>"
export GATEWAY="<value>"
export API="<value>"
export WS="<value>"
```

```powershell
# Windows PowerShell
$env:ORIGIN = "<value>"
$env:API_KEY = "<value>"
$env:GATEWAY = "<value>"
$env:API = "<value>"
$env:WS = "<value>"
```

**Build the binary:**

```bash
cargo build --release
# or
./run.sh build
```

The binary is placed at `target/release/teneo-bot` (`teneo-bot.exe` on Windows).

---

## Configuration

### 1. Accounts (.env)

Create `.env` in the project root with one account per line. Each key must start with `ACCOUNTS_` and the value uses the `email|token|node_count` format. Blank lines and lines starting with `#` are ignored. Keep the file private:

```
ACCOUNTS_1=you@example.com|yourtoken|2
ACCOUNTS_2=another@example.com|anothertoken|1
```

```bash
chmod 600 .env
```

`node_count` is how many node keys the account should hold. A value below 1 or a value that is not a number counts as 1. Accounts are processed in alphabetical order of the email.

### 2. Node Cache (nodeId.json)

`nodeId.json` stores the node keys that were issued for each account. It is created and updated by the bot, so an empty file or no file at all is fine on the first run:

```json
{}
```

Every key that was issued is kept, so a later run reuses the same nodes. If you raise `node_count` in `.env`, only the missing keys are issued. Keep this file private and back it up, because the keys inside it cannot be recovered from the bot.

### 3. Proxy (proxy.txt)

Fill `proxy.txt` with a proxy (optional, leave empty to run without proxy). Blank lines and lines starting with `#` are ignored and only the first usable line is read:

```
http://user:pass@host:port
socks5://user:pass@host:port
```

Supported schemes are `http`, `https`, `socks5`, `socks5h` and `socks`. A line without a scheme is read as `socks5`, and a line without a port uses port `1080`. Special characters in the username or password can be written with percent encoding, for example `%40` for `@`.

The proxy is used for the API calls. The relay sockets go out directly and do not use the proxy.

### 4. Bot Settings (config.json)

`config.json` must exist next to the binary. If a key is missing, its default is used.

```json
{
  "settings": {
    "sleep_seconds": 900
  }
}
```

---

## First-Time Setup

**Step 1 - Put the accounts in `.env`** using the format above.

**Step 2 - Build once:**

```bash
./run.sh build
```

**Step 3 - Check the local files:**

```bash
./run.sh selftest
```

This checks the frame codec, the proxy mask, the node cache and that `.env` holds at least one account. It does not touch the network.

**Step 4 - Run the loop once to issue the node keys:**

```bash
./run.sh run
```

Node keys are issued on the first run of the loop, and the loop also sends heartbeats and claims when the boost is due. After the keys exist, the read only check can be used at any time:

```bash
./run.sh probe
```

`probe` skips any account that has no node key yet, so run the loop at least once before relying on it.

---

## Running the Bot

Make `run.sh` executable first:

```bash
chmod +x run.sh
```

Then choose a mode:

```bash
./run.sh run        # build if needed and run in foreground
./run.sh bg         # build if needed and run in a detached screen session
./run.sh screen     # attach to the running screen session
./run.sh stop       # stop the screen session
./run.sh probe      # read state, fleet, and points without claiming anything
./run.sh selftest   # local checks, no network
./run.sh build      # build only, do not run
```

The background mode uses a screen session named `teneo`. Attach anytime with `./run.sh screen` and detach with `Ctrl+A` then `D`.

Press `Ctrl+C` once to stop the bot cleanly when running in foreground.

### Using make (Linux / macOS)

```bash
make build      # optimized release build
make run        # build release and run the loop
make probe      # build release and run the read only probe
make selftest   # build release and run the local checks
make fmt        # format the code
make clean      # remove all build artifacts
make dist       # build release and copy the binary into dist/
```

### Manual run

```bash
# Linux / macOS / Termux
./target/release/teneo-bot

# Windows
.\target\release\teneo-bot.exe
```

Run the binary from the folder that holds `.env`, `nodeId.json`, `proxy.txt` and `config.json`.

---

## Run Modes

| Mode | Command | Description |
|---|---|---|
| run | `./run.sh run` | Full loop: node keys, relay sockets, heartbeat, claim, sleep |
| bg | `./run.sh bg` | Same as run in a detached screen session |
| screen | `./run.sh screen` | Attach to the background session |
| probe | `./run.sh probe` | Read state, fleet, and points without claiming |
| selftest | `./run.sh selftest` | Local checks, no network |
| stop | `./run.sh stop` | Stop the background session |
| build | `./run.sh build` | Build only |

The binary also accepts `loop` as another name for `run`. An empty or unknown mode starts the loop.

---

## Packaging

`package.sh` builds the release binary and writes `dist/TeneoBeacon.zip`, a clean copy of the project with the sources, `Cargo.toml`, `Cargo.lock`, `build.rs`, `Makefile`, `run.sh`, `config.json`, the icons and the CI workflow. It also creates a fresh `nodeId.json`, `proxy.txt` and `.env` with comment lines only, so no account or key leaves your machine.

```bash
chmod +x package.sh
./package.sh
```

The five build variables from the Installation section must be set before running it.

---

## Download Prebuilt Binary

Prebuilt binaries are compiled by GitHub Actions on every push to `main` or `master`, on every `v*` tag, and on manual runs, for Windows and macOS. When a `v*` tag is pushed, the binaries are also attached to the release.

Download the latest binaries from the Actions page:
https://github.com/Yuurisan-N1/Teneo-Beacon/actions/workflows/build.yml

Open the latest successful run and scroll to the Artifacts section.

| Artifact | Platform |
|---|---|
| `teneo-bot-windows-x86_64` | Windows x86_64 |
| `teneo-bot-macos-aarch64` | macOS Apple Silicon |

**macOS after downloading:**

```bash
chmod +x teneo-bot-macos-aarch64
./teneo-bot-macos-aarch64 run
```

**Windows:**

```bash
.\teneo-bot-windows-x86_64.exe run
```

Place the binary in the same folder as `.env`, `nodeId.json`, `proxy.txt`, and `config.json` before running. Linux users build from source.

---

## Features

### Node Keys
For every account, the bot checks `nodeId.json` and registers new node keys with the gateway until the count set in `.env` is reached. Each new key is written to the cache right away through a temporary file and a rename, so an interrupted run does not corrupt it. If a key cannot be issued, the cache stays as it is and the cycle continues with the keys that exist.

### Relay Channels
The bot opens one websocket to the gateway for every node key and keeps it open. Each channel announces itself with the account token and the account email, then serves the connect, data, and close frames the gateway sends, opening the requested TCP connection and passing data in both directions. Targets that resolve to private, loopback, link-local, multicast, or unspecified addresses are skipped. Points and fragment updates pushed by the gateway are logged as they arrive.

If a channel drops, it reconnects with a delay that starts at 5 seconds, doubles on each failure, and stops growing at 300 seconds. If the gateway closes the channel with code `4001` or `4005`, the channel waits 30 minutes before trying again. The first failure of a channel is logged and repeated failures stay quiet until it reconnects.

### Auto Heartbeat
One heartbeat loop runs for each account. It sends the heartbeat right away and then repeats every `heartbeat_minutes`. The points earned today and the running total are logged on success, and a refused heartbeat is logged with the reason.

### Auto Beacon Claim
On every cycle the bot reads the beacon state for the account. If the next boost time is still in the future, the claim is skipped and the fragments stay unclaimed. If there are no unclaimed fragments, the step is logged and skipped. Otherwise each node requests a challenge token and submits the boost claim, logging the fragments credited and the beacon power. The loop stops as soon as no fragments remain, and a summary shows the total fragments claimed and how many nodes were used. A claim that is still on cooldown is logged as such.

### Beacon Report
After the claim step, the bot logs the unclaimed fragments, the beacon power and the number of nodes online, then the fleet summary with nodes online out of the total and the fragments earned. It also logs how many relay channels are open out of the expected count, and warns when the gateway reports no active node for the account.

### Probe Mode
`probe` reads the state, the fleet, and the points for each account using its first node key and never sends a claim or a heartbeat. Accounts without a node key are skipped.

### Self Test
`selftest` runs a round trip through the frame encoder and decoder, checks that the proxy mask hides the password, writes and reads a temporary node cache file that is deleted afterward, and confirms that `.env` holds at least one account.

### Proxy Support
The proxy from `proxy.txt` is applied to the API requests. It is shown in masked form on startup and credentials are never printed. Running without a proxy is fully supported.

### Multi Account
All accounts in `.env` are processed sequentially within every cycle, and a blank line separates each account output in the terminal for readability. The relay sockets and heartbeat loops are started once in the first cycle and keep running in the background while later cycles repeat the claim and report steps.

### Auto Countdown
After all accounts complete a cycle, the bot displays a live countdown in the terminal until the next cycle starts, then re-shows the banner before beginning again. When the output is not a terminal, the wait is logged once instead.

---

## Troubleshooting

| Message | Cause | Fix |
|---|---|---|
| `the environment file .env could not be read` | `.env` does not exist | Create `.env` in the project root |
| `No account was found in the environment file` | No line starts with `ACCOUNTS_` | Add lines in `ACCOUNTS_n=email\|token\|node_count` format |
| `the environment line ACCOUNTS_n is missing a field` | A line has fewer than three fields | Check the `email\|token\|node_count` format |
| `the config file config.json could not be read` | `config.json` is missing | Restore `config.json` next to the binary |
| `the config file is not valid json` | A syntax error in `config.json` | Fix the JSON |
| `proxy scheme ... is not supported` | Unknown scheme in `proxy.txt` | Use `http`, `https`, or `socks5` |
| `the proxy port is not a number` | Bad port in `proxy.txt` | Check the `host:port` part |
| `the node credential was refused` | The token or node key was rejected | Check the token in `.env` and the key in `nodeId.json` |
| `the firewall refused the request` | The server answered 403 | Change the proxy or the network |
| `the server is limiting requests from this address` | Too many requests | Raise `sleep_seconds` or change the proxy |
| `the beacon gateway is not answering right now` | The gateway returned a 5xx status | Wait and try again |
| `The claim is not due yet so the fragments stay unclaimed` | The boost time has not arrived | Wait for the next cycle |
| `the claim is still on cooldown` | The gateway refused a claim during cooldown | Wait for the next cycle |
| `The gateway reports no active node for this account` | No relay channel is connected | Check that the relay channels opened, the network, and the node keys |
| `The relay channel number N could not be opened` | The websocket failed to connect | The bot retries on its own with a growing delay |

---

## File Structure

```text
Teneo-Beacon/
├── .github/
│   └── workflows/
│       └── build.yml                    # CI: Windows and macOS build, release on v* tags
├── src/
│   ├── main.rs                          # Entry point, runtime, shared context, mode dispatch
│   ├── beacon/
│   │   ├── state.rs                     # Unclaimed fragments, power, nodes online, next boost time
│   │   ├── fleet.rs                     # Fleet summary: online, total, fragments, uptime
│   │   ├── challenge.rs                 # Challenge token request
│   │   ├── boost.rs                     # Boost claim
│   │   └── mod.rs
│   ├── cli/
│   │   ├── dispatch.rs                  # Mode selector: run, probe, selftest
│   │   ├── probe.rs                     # Read only probe
│   │   ├── selftest.rs                  # Local checks
│   │   └── mod.rs
│   ├── net/
│   │   ├── client.rs                    # HTTP client, headers, JSON helpers
│   │   ├── endpoint.rs                  # API endpoint definitions
│   │   ├── tunnel.rs					 # HTTP CONNECT/SOCKS5 tunnels and TLS client configuration
│   │   ├── proxy.rs                     # Proxy parser, loader, and masker
│   │   └── mod.rs
│   ├── node/
│   │   ├── pool.rs                      # Node key pool per account
│   │   ├── register.rs                  # Node key registration
│   │   └── mod.rs
│   ├── points/
│   │   ├── heartbeat.rs                 # Account heartbeat
│   │   ├── streak.rs                    # Points total and daily points
│   │   └── mod.rs
│   ├── relay/
│   │   ├── stream.rs                    # Websocket channel and frame serving
│   │   ├── frames.rs                    # Frame encoder and decoder
│   │   ├── socket.rs                    # TCP dial with blocked address check
│   │   └── mod.rs
│   ├── run/
│   │   ├── cycle.rs                     # Main loop, relay and heartbeat tasks, claim, countdown
│   │   └── mod.rs
│   └── support/
│       ├── banner.rs                    # ASCII banner
│       ├── cfg/                         # Config loader and schema
│       ├── consts/                      # App and network constants
│       ├── env.rs                       # Account loader for .env
│       ├── error.rs                     # Error messages and status labels
│       ├── log/                         # Log init, colored format, countdown helper
│       ├── signal.rs                    # Ctrl+C handler
│       ├── store.rs                     # Node cache read and write
│       └── mod.rs
├── assets/
│   ├── icon.ico                         # Windows binary icon
│   └── icon.png                         # App icon
├── dist/                                # Output of package.sh and make dist (generated, gitignored)
├── Cargo.toml                           # Project manifest and dependencies
├── Cargo.lock
├── build.rs                             # Build script (embeds icon into Windows binary)
├── Makefile                             # make targets: build, run, probe, selftest, fmt, clean, dist
├── run.sh                               # Run helper: build, run, probe, selftest, bg, screen, stop
├── package.sh                           # Builds and writes a clean dist/TeneoBeacon.zip
├── config.json                          # Sleep duration and heartbeat interval
├── .env                                 # Account lines (create it yourself, gitignored)
├── .env.example                         # Account line template
├── nodeId.json                          # Node key cache per account (auto-generated, gitignored)
├── proxy.txt                            # Proxy (optional)
├── .gitignore
├── HOWTO.txt                            # Quick start guide
└── LICENSE                              # GNU AGPL v3.0
```

---

## Disclaimer

This tool is built for educational and technical exploration purposes. Use it wisely and at your own responsibility.

---

<div align="center">
<img width="100%" alt="footer" src="https://capsule-render.vercel.app/api?type=waving&height=120&section=footer"/>
</div>