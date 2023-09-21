## Examples

To build these ensure you are in the `examples-esp32XXX` directory matching your target as othewise the `config.toml` will not apply

### dhcp

- set SSID and PASSWORD env variable
- gets an ip address via DHCP
- performs an HTTP get request to some "random" server

`cargo run --example dhcp --release --features "embedded-svc,wifi"`

### static_ip

- set SSID and PASSWORD env variable
- set STATIC_IP and GATEWAY_IP env variable (e.g. "192.168.2.191" / "192.168.2.1")
- might be necessary to configure your WiFi access point accordingly
- uses the given static IP
- responds with some HTML content when connecting to port 8080

`cargo run --example static_ip --release --features "embedded-svc,wifi"`

### ble

- starts Bluetooth advertising
- offers one service with three characteristics (one is read/write, one is write only, one is read/write/notify)
- pressing the boot-button on a dev-board will send a notification if it is subscribed
- this uses a toy level BLE stack - might not work with every BLE central device (tested with Android and Windows Bluetooth LE Explorer)

`cargo run --example ble --release --features "ble"`

**NOTE:** ESP32-S2 doesn't support bluetooth

### async_ble

- same as `ble` but async

`cargo run --example async_ble --release --features "async,ble"`

**NOTE:** ESP32-S2 doesn't support bluetooth

### coex

- set SSID and PASSWORD env variable
- gets an ip address via DHCP
- performs an HTTP get request to some "random" server
- does BLE advertising
- coex support is still somewhat flaky

`cargo run --example coex --release --features "embedded-svc,wifi,ble"`

**NOTE:** Not currently available for the ESP32, ESP32-C2, ESP32-C6 or ESP32-S2

### esp_now

- broadcasts, receives and sends messages via esp-now

`cargo run --example esp_now --release --features "esp-now"`

### embassy_esp_now

- broadcasts, receives and sends messages via esp-now in an async way

`cargo run --example embassy_esp_now --release --features "async,esp-now"`

### embassy_esp_now_duplex

- asynchronously broadcasts, receives and sends messages via esp-now in multiple embassy tasks

`cargo run --example embassy_esp_now_duplex --release --features "async,esp-now"`

### embassy_dhcp

- Read and Write to sockets over WiFi asyncronously using embassy-executor.

`cargo run --example embassy_dhcp --release --features "async,embedded-svc,wifi,embassy-net"`

### access_point

- creates an open access-point with SSID `esp-wifi`
- you can connect to it using a static IP in range 192.168.2.2 .. 192.168.2.255, gateway 192.168.2.1
- open http://192.168.2.1:8080/ in your browser
- on Android you might need to choose _Keep Accesspoint_ when it tells you the WiFi has no internet connection, Chrome might not want to load the URL - you can use a shell and try `curl` and `ping`

`cargo run --example access_point --release --features "embedded-svc,wifi"`

### embassy_access_point

- creates an open access-point with SSID `esp-wifi`
- you can connect to it using a static IP in range 192.168.2.2 .. 192.168.2.255, gateway 192.168.2.1
- open http://192.168.2.1:8080/ in your browser
- on Android you might need to choose _Keep Accesspoint_ when it tells you the WiFi has no internet connection, Chrome might not want to load the URL - you can use a shell and try `curl` and `ping`

`cargo run --example embassy_access_point --release --features "async,embedded-svc,wifi,embassy-net"`