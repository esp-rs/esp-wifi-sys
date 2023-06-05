REM A simple script to help in smoke-testing.
REM Not really useful to users.

@echo off
echo Make sure to have the env-vars SSID, PASSWORD, STATIC_IP and GATEWAY_IP set.
echo Use CTRL-C to exit an example and start the next one.

cd examples-esp32c3
set CARGO_PROFILE_RELEASE_OPT_LEVEL=3
set CARGO_PROFILE_RELEASE_LTO=off
echo.
echo Connect ESP32-C3
pause
cargo run --example ble --release --features "ble"
cargo run --example async_ble --release --features "async,ble"
cargo run --example dhcp --release --features "wifi"
cargo run --example static_ip --release --features "wifi"
cargo run --example embassy_dhcp --release --features "async,wifi,embassy-net"
cargo run --example coex --release --features "wifi,ble,coex"
cargo run --example esp_now --release --features "esp-now"
cargo run --example embassy_esp_now --release --features "async,esp-now"
cargo run --example access_point --release --features "wifi"
cargo run --example embassy_access_point --release --features "async,wifi,embassy-net"

cd ..\examples-esp32
set CARGO_PROFILE_RELEASE_OPT_LEVEL=3
set CARGO_PROFILE_RELEASE_LTO=off
echo.
echo Connect ESP32
pause
cargo run --example ble --release --features "ble"
cargo run --example async_ble --release --features "async,ble"
cargo run --example dhcp --release --features "wifi"
cargo run --example static_ip --release --features "wifi"
cargo run --example embassy_dhcp --release --features "async,wifi,embassy-net"
cargo run --example esp_now --release --features "esp-now"
cargo run --example embassy_esp_now --release --features "async,esp-now"
cargo run --example access_point --release --features "wifi"
cargo run --example embassy_access_point --release --features "async,wifi,embassy-net"

cd ..\examples-esp32s3
set CARGO_PROFILE_RELEASE_OPT_LEVEL=3
set CARGO_PROFILE_RELEASE_LTO=off
echo.
echo Connect ESP32-S3
pause
cargo run --example ble --release --features "ble"
cargo run --example async_ble --release --features "async,ble"
cargo run --example dhcp --release --features "wifi"
cargo run --example static_ip --release --features "wifi"
cargo run --example embassy_dhcp --release --features "async,wifi,embassy-net"
cargo run --example coex --release --features "wifi,ble,coex"
set CARGO_PROFILE_RELEASE_OPT_LEVEL=1
cargo run --example esp_now --release --features "esp-now"
cargo run --example embassy_esp_now --release --features "async,esp-now"
set CARGO_PROFILE_RELEASE_OPT_LEVEL=3
cargo run --example access_point --release --features "wifi"
cargo run --example embassy_access_point --release --features "async,wifi,embassy-net"

cd ..\examples-esp32s2
set CARGO_PROFILE_RELEASE_OPT_LEVEL=2
set CARGO_PROFILE_RELEASE_LTO=off
echo.
echo Connect ESP32-S2
pause
cargo run --example dhcp --release --features "wifi"
cargo run --example static_ip --release --features "wifi"
cargo run --example embassy_dhcp --release --features "async,wifi,embassy-net"
cargo run --example esp_now --release --features "esp-now"
cargo run --example embassy_esp_now --release --features "async,esp-now"
cargo run --example access_point --release --features "wifi"
cargo run --example embassy_access_point --release --features "async,wifi,embassy-net"

cd ..\examples-esp32c2
set CARGO_PROFILE_RELEASE_OPT_LEVEL=3
set CARGO_PROFILE_RELEASE_LTO=false
echo.
echo Connect ESP32-C2
pause
cargo run --example ble --release --features "ble"
cargo run --example async_ble --release --features "async,ble"
cargo run --example dhcp --release --features "wifi"
cargo run --example static_ip --release --features "wifi"
cargo run --example embassy_dhcp --release --features "async,wifi,embassy-net"
cargo run --example esp_now --release --features "esp-now"
cargo run --example embassy_esp_now --release --features "async,esp-now"
cargo run --example access_point --release --features "wifi"
cargo run --example embassy_access_point --release --features "async,wifi,embassy-net"

cd ..\examples-esp32c6
set CARGO_PROFILE_RELEASE_OPT_LEVEL=3
set CARGO_PROFILE_RELEASE_LTO=off
echo.
echo Connect ESP32-C6
pause
cargo run --example dhcp --release --features "wifi"
cargo run --example static_ip --release --features "wifi"
cargo run --example embassy_dhcp --release --features "async,wifi,embassy-net"
cargo run --example esp_now --release --features "esp-now"
cargo run --example embassy_esp_now --release --features "async,esp-now"
cargo run --example access_point --release --features "wifi"
cargo run --example embassy_access_point --release --features "async,wifi,embassy-net"
