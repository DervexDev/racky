<div align="center">
  <img alt="Logo" width="256" src=".github/logo.svg"/>
</div>

# Racky

Racky is a simple tool for managing, running and monitoring programs on your home server(s) with ease!

## Features

Here are some key features of Racky:

- 📦 **Program management**
  - Upload and remove binaries or scripts (_programs_) from any computer
  - Run, stop, restart your programs with a single command
  - Configure program lifecycle and your custom environment variables
  - Check program status and logs via the CLI or web
- 🖥️ **Server management**
  - Add, change and remove as many server definitions as you want
  - Restart, reboot and update your server remotely
  - Configure server behavior and defaults with a config file or CLI
  - View server logs and status via the CLI or web
- 🚀 **Performant all-in-one solution**
  - All that and more you can do either from client or server using a single `racky` command
  - Written in Rust meaning impact on system performance and resource usage is minimal

## Installation

To install Racky client/server simply make sure you meet the requirements and run the command for your platform.

### Client

Install the client on all your computers from which you want to manage your server(s).

#### macOS & Linux

Run the following command in your terminal:

```bash
curl -sSL https://raw.githubusercontent.com/DervexDev/racky/main/scripts/install.sh | bash
```

> Requires `bash`, `curl`, `unzip`, `uname` and `tr` commands.

#### Windows

Run the following command in PowerShell:

```powershell
Invoke-RestMethod https://raw.githubusercontent.com/DervexDev/racky/main/scripts/install.ps1 | Invoke-Expression
```

### Server

Install the server on your actual server machine(s).

#### Linux

Run the following command in your terminal:

```bash
curl -sSL https://raw.githubusercontent.com/DervexDev/racky/main/scripts/install.sh | sudo bash -s -- -s
```

> Requires `sudo` `bash`, `curl`, `unzip`, `uname`, `tr` commands and a **systemd** based Linux distro.

## Quick Start

Before proceeding, make sure you have already installed Racky client on your personal computer and Racky server on your home server(s).

### 1. Start a Racky server on your server machine

If the installation process completed successfully, Racky server should be already running. Otherwise, run:

```bash
racky server start
```

> While you are on your server machine, run `ip addr show` command to check your server's local IP address.

### 2. Add the server to your Racky client on your personal computer

To register the server named `optiplex` with address `192.168.1.69`, run the following command:

```bash
racky server add optiplex --address 192.168.1.69
```

> By default `--port` is `5000` and `--password` is an empty string, unless changed in the server configuration.

### 3. Upload and start a new program on the server

In order to add and start a new program on the server (named after the path target), run this command:

```bash
racky program add path/to/my-app --auto-start
```

> Path must point to a valid Linux executable or a directory containing a `racky.sh` or `scripts/racky.sh` script.

### 4. Check the program status

To verify if the program got installed and started successfully, run the following command:

```bash
racky program status my-app
```

> If you want to target a specific server e.g. `optiplex`, add the `--server optiplex` flag.

## Usage

Run the following command in your terminal to learn about Racky:

### Main commands

```bash
racky -h
```

### Program subcommands

```bash
racky program -h
```

### Server subcommands

```bash
racky server -h
```

## Configuration

Racky allows you to configure both client and server as well as each program you add.

### Updating the config

To set `key1` to `value1` and reset `key2` to its default value, run this command:

```bash
racky config key1=value1 key2=
```

> Updating server/program configs and program environment variables is done the same way.

### Available settings

You cen check all available settings and its default/current values by running:

```bash
racky config --list
```

> This flag is also available for server and program configurations.
