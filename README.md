# LED Matrix Daemon for Framework 16" Laptops

This repository contains a daemon for controlling the LED matrix on Framework 16" laptops. The daemon enables you to
send images to be displayed on the LED matrix.

## Features

- **Daemon Service**: Operates in the background, listening for commands to update the LED matrix.
- **Keep Port Open**: Maintains the port connection without closing it (the daemon manages the port).
- **Port Timeout**: Defaults to 2 seconds, though initial wake-open may take around 1 second.
- **Retry on Port Failure**: Continually retries to open the port if it fails initially.
- **Port Swap**: Allows specification of the left port that is actually on the left side.
- **Unix Socket Listener**: Supports Unix socket connections.
- **TCP Listener**: Supports TCP connections.

## Requirements

- Framework 16" laptop
- At least one LED matrix

## Installation

### Arch Linux

```bash
yay -S led_matrix_daemon
```

Enable daemon with default configuration:

```bash
sudo systemctl enable --now led_matrix_daemon.socket led_matrix_daemon.service
```

Configuration is located at `/etc/led_matrix/daemon.toml`.

### Build

Install Rust:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Check out the repository and build the binary:

```bash
git clone https://github.com/night-crawler/led_matrix_daemon.git
cd led_matrix_daemon
cargo build --release
```

Copy the binary to a location in your path:

```bash
sudo cp ./target/release/led_matrix_daemon /usr/local/bin
```

### Run Example Configuration

```bash
./target/release/led_matrix_daemon -c ./test_data/config.toml
```

In a different terminal, run the following scripts to trigger LED animations:

```bash
./test_data/curl_test_file.sh
```

```bash
./test_data/curl_test_b64.sh
```

```bash
./test_data/curl_test_b64_multiple.sh
```

## Configuration

Configuration sample:

```toml
listen_address = "127.0.0.1:45935"
unix_socket = "/tmp/led-matrix.sock"

max_queue_size = 10
num_http_workers = 4

[left_port]
path = "/dev/ttyACM0"
baud_rate = 115200
timeout = "2s"
keep_open = true

[right_port]
path = "/dev/ttyACM1"
baud_rate = 115200
timeout = "2s"
keep_open = true
wait_delay = "1s"
```

## Usage

This daemon provides two endpoints: one for multipart form data and another for base64-encoded images.

The multipart endpoint works as follows: if there is only one port available, all data sent will be transferred to that
port. If two ports are available, the data will be distributed based on even-odd positioning: even-numbered data goes to
the left port, and odd-numbered data goes to the right port.

- [Base64 mode single](test_data/curl_test_b64.sh)
- [Base64 mode multiple](test_data/curl_test_b64_multiple.sh)
- [File mode](test_data/curl_test_file.sh)

Remember, the size of the image must be 9x34.
The daemon does not resize anything, it's deliberately stupid.
