REM A simple script to help in smoke-testing.
REM Not really useful to users.

@echo off
echo Make sure to have the env-vars SSID, PASSWORD, STATIC_IP and GATEWAY_IP set.
echo Use CTRL-C to exit an example and start the next one.

cd esp-wifi

set CARGO_PROFILE_RELEASE_OPT_LEVEL=3
set CARGO_PROFILE_RELEASE_LTO=off
echo.
echo Connect ESP32-C3
pause
cargo +nightly esp32c3 --example ble --release --features "ble"
cargo +nightly esp32c3 --example embassy_ble --release --features "async,ble"
cargo +nightly esp32c3 --example dhcp --release --features "wifi"
cargo +nightly esp32c3 --example static_ip --release --features "wifi"
cargo +nightly esp32c3 --example embassy_dhcp --release --features "async,wifi,embassy-net"
cargo +nightly esp32c3 --example coex --release --features "wifi,ble,coex"
cargo +nightly esp32c3 --example esp_now --release --features "esp-now"
cargo +nightly esp32c3 --example embassy_esp_now --release --features "async,esp-now"
cargo +nightly esp32c3 --example access_point --release --features "wifi"
cargo +nightly esp32c3 --example embassy_access_point --release --features "async,wifi,embassy-net"

set CARGO_PROFILE_RELEASE_OPT_LEVEL=3
set CARGO_PROFILE_RELEASE_LTO=off
echo.
echo Connect ESP32
pause
cargo +esp esp32 --example ble --release --features "ble"
cargo +esp esp32 --example embassy_ble --release --features "async,ble"
cargo +esp esp32 --example dhcp --release --features "wifi"
cargo +esp esp32 --example static_ip --release --features "wifi"
cargo +esp esp32 --example embassy_dhcp --release --features "async,wifi,embassy-net"
cargo +esp esp32 --example coex --release --features "wifi,ble,coex"
cargo +esp esp32 --example esp_now --release --features "esp-now"
cargo +esp esp32 --example embassy_esp_now --release --features "async,esp-now"
cargo +esp esp32 --example access_point --release --features "wifi"
cargo +esp esp32 --example embassy_access_point --release --features "async,wifi,embassy-net"

set CARGO_PROFILE_RELEASE_OPT_LEVEL=3
set CARGO_PROFILE_RELEASE_LTO=off
echo.
echo Connect ESP32-S3
pause
cargo +esp esp32s3 --example ble --release --features "ble"
cargo +esp esp32s3 --example embassy_ble --release --features "async,ble"
cargo +esp esp32s3 --example dhcp --release --features "wifi"
cargo +esp esp32s3 --example static_ip --release --features "wifi"
cargo +esp esp32s3 --example embassy_dhcp --release --features "async,wifi,embassy-net"
cargo +esp esp32s3 --example coex --release --features "wifi,ble,coex"
cargo +esp esp32s3 --example esp_now --release --features "esp-now"
cargo +esp esp32s3 --example embassy_esp_now --release --features "async,esp-now"
cargo +esp esp32s3 --example access_point --release --features "wifi"
cargo +esp esp32s3 --example embassy_access_point --release --features "async,wifi,embassy-net"

set CARGO_PROFILE_RELEASE_OPT_LEVEL=2
set CARGO_PROFILE_RELEASE_LTO=off
echo.
echo Connect ESP32-S2
pause
cargo +esp esp32s2 --example dhcp --release --features "wifi"
cargo +esp esp32s2 --example static_ip --release --features "wifi"
cargo +esp esp32s2 --example embassy_dhcp --release --features "async,wifi,embassy-net"
cargo +esp esp32s2 --example esp_now --release --features "esp-now"
cargo +esp esp32s2 --example embassy_esp_now --release --features "async,esp-now"
cargo +esp esp32s2 --example access_point --release --features "wifi"
cargo +esp esp32s2 --example embassy_access_point --release --features "async,wifi,embassy-net"

set CARGO_PROFILE_RELEASE_OPT_LEVEL=3
set CARGO_PROFILE_RELEASE_LTO=false
echo.
echo Connect ESP32-C2
pause
cargo +nightly esp32c2 --example ble --release --features "ble"
cargo +nightly esp32c2 --example embassy_ble --release --features "async,ble"
cargo +nightly esp32c2 --example dhcp --release --features "wifi"
cargo +nightly esp32c2 --example static_ip --release --features "wifi"
cargo +nightly esp32c2 --example embassy_dhcp --release --features "async,wifi,embassy-net"
cargo +nightly esp32c2 --example coex --release --features "wifi,ble,coex,big-heap"
cargo +nightly esp32c2 --example esp_now --release --features "esp-now"
cargo +nightly esp32c2 --example embassy_esp_now --release --features "async,esp-now"
cargo +nightly esp32c2 --example access_point --release --features "wifi"
cargo +nightly esp32c2 --example embassy_access_point --release --features "async,wifi,embassy-net"

set CARGO_PROFILE_RELEASE_OPT_LEVEL=3
set CARGO_PROFILE_RELEASE_LTO=off
echo.
echo Connect ESP32-C6
pause
cargo +nightly esp32c6 --example ble --release --features "ble"
cargo +nightly esp32c6 --example embassy_ble --release --features "async,ble"
cargo +nightly esp32c6 --example dhcp --release --features "wifi"
cargo +nightly esp32c6 --example static_ip --release --features "wifi"
cargo +nightly esp32c6 --example embassy_dhcp --release --features "async,wifi,embassy-net"
cargo +nightly esp32c6 --example coex --release --features "wifi,ble,coex,big-heap"
cargo +nightly esp32c6 --example esp_now --release --features "esp-now"
cargo +nightly esp32c6 --example embassy_esp_now --release --features "async,esp-now"
cargo +nightly esp32c6 --example access_point --release --features "wifi"
cargo +nightly esp32c6 --example embassy_access_point --release --features "async,wifi,embassy-net"
