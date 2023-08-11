# Tuning the configuration

You can change a few of the default settings used. Please keep in mind that it's almost always a tradeoff between memory usage and performance.
It's easy to decrease performance by using unfortunate settings or even break the application or making it less stable.

So you should test every change very carefully.

Be aware that settings which work fine on one ESP32 model might not work at all on other. Also, settings should be adjusted according to your application's needs.

## Create a configuration

We use [toml-cfg](https://crates.io/crates/toml-cfg) for the build time configuration

You need to add a `cfg.toml` file in the root of your binary crate. When using a _Cargo Workspace_ you should put the file in the root of the workspace directory.

A configuration file can look like this:
```toml
[esp-wifi]
rx_queue_size = 15
tx_queue_size = 3
static_rx_buf_num = 10
dynamic_rx_buf_num = 16
ampdu_rx_enable = 0
ampdu_tx_enable = 0
rx_ba_win = 32
max_burst_size = 6
```

You can set the following settings
|Key|Description|
|-|-|
|rx_queue_size|Size of the RX queue in frames|
|tx_queue_size|Size of the TX queue in frames|
|max_burst_size|See [documentation](https://docs.rs/smoltcp/0.10.0/smoltcp/phy/struct.DeviceCapabilities.html#structfield.max_burst_size)|
|static_rx_buf_num|WiFi static RX buffer number. See [ESP-IDF Programming Guide](https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-reference/network/esp_wifi.html#_CPPv418wifi_init_config_t)|
|dynamic_rx_buf_num|WiFi dynamic RX buffer number. See [ESP-IDF Programming Guide](https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-reference/network/esp_wifi.html#_CPPv418wifi_init_config_t)|
|static_tx_buf_num|WiFi static TX buffer number. See [ESP-IDF Programming Guide](https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-reference/network/esp_wifi.html#_CPPv418wifi_init_config_t)|
|dynamic_tx_buf_num|WiFi dynamic TX buffer number. See [ESP-IDF Programming Guide](https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-reference/network/esp_wifi.html#_CPPv418wifi_init_config_t)|
|ampdu_rx_enable|WiFi AMPDU RX feature enable flag. See [ESP-IDF Programming Guide](https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-reference/network/esp_wifi.html#_CPPv418wifi_init_config_t)|
|ampdu_rx_enable|WiFi AMPDU RX feature enable flag. (0 or 1) See [ESP-IDF Programming Guide](https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-reference/network/esp_wifi.html#_CPPv418wifi_init_config_t)|
|ampdu_tx_enable|WiFi AMPDU TX feature enable flag. (0 or 1) See [ESP-IDF Programming Guide](https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-reference/network/esp_wifi.html#_CPPv418wifi_init_config_t)|
|amsdu_tx_enable|WiFi AMSDU TX feature enable flag. (0 or 1) See [ESP-IDF Programming Guide](https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-reference/network/esp_wifi.html#_CPPv418wifi_init_config_t)|
|rx_ba_win|WiFi Block Ack RX window size. See [ESP-IDF Programming Guide](https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-reference/network/esp_wifi.html#_CPPv418wifi_init_config_t)|
|country_code|Country code. See [ESP-IDF Programming Guide](https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-guides/wifi.html#wi-fi-country-code)|
|country_code_operating_class|If not 0: Operating Class table number. See [ESP-IDF Programming Guide](https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-guides/wifi.html#wi-fi-country-code)|
|mtu|MTU, see [documentation](https://docs.rs/smoltcp/0.10.0/smoltcp/phy/struct.DeviceCapabilities.html#structfield.max_transmission_unit)|
|heap_size|Size of the WiFi/BLE heap in bytes|
|tick_rate_hz|Tick rate of the internal task scheduler in hertz.|

## Globally disable logging

`esp-wifi` contains a lot of trace-level logging statements. For maximum performance you might want to disable logging via a feature flag of the `log` crate. See [documentation](https://docs.rs/log/0.4.19/log/#compile-time-filters). You should set it to `release_max_level_off`
