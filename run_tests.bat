mkdir tmp
mkdir tmp\esp32
mkdir tmp\esp32s2
mkdir tmp\esp32s3
mkdir tmp\esp32c2
mkdir tmp\esp32c3
mkdir tmp\esp32c6

cd esp-wifi
cargo +esp build --release --example esp_now_broadcaster --target xtensa-esp32-none-elf --features esp32,esp32-hal/default,esp32-hal/embassy-time-timg0,esp32-hal/embassy-executor-thread,wifi,esp-now
cargo +esp build --release --example test_esp_now --target xtensa-esp32-none-elf --features esp32,esp32-hal/default,esp32-hal/embassy-time-timg0,esp32-hal/embassy-executor-thread,wifi,esp-now
cargo +esp build --release --example open_access_point --target xtensa-esp32-none-elf --features esp32,esp32-hal/default,esp32-hal/embassy-time-timg0,esp32-hal/embassy-executor-thread,wifi,esp-now
cargo +esp build --release --example test_connect --target xtensa-esp32-none-elf --features esp32,esp32-hal/default,esp32-hal/embassy-time-timg0,esp32-hal/embassy-executor-thread,wifi,esp-now
cargo +esp build --release --example test_ble --target xtensa-esp32-none-elf --features esp32,esp32-hal/default,esp32-hal/embassy-time-timg0,esp32-hal/embassy-executor-thread,ble
copy ..\target\xtensa-esp32-none-elf\release\examples\esp_now_broadcaster ..\tmp\esp32
copy ..\target\xtensa-esp32-none-elf\release\examples\test_esp_now ..\tmp\esp32
copy ..\target\xtensa-esp32-none-elf\release\examples\open_access_point ..\tmp\esp32
copy ..\target\xtensa-esp32-none-elf\release\examples\test_connect ..\tmp\esp32
copy ..\target\xtensa-esp32-none-elf\release\examples\test_ble ..\tmp\esp32

cargo +esp build --release --example esp_now_broadcaster --target xtensa-esp32s2-none-elf --features esp32s2,esp32s2-hal/default,esp32s2-hal/embassy-time-timg0,esp32s2-hal/embassy-executor-thread,wifi,esp-now
cargo +esp build --release --example test_esp_now --target xtensa-esp32s2-none-elf --features esp32s2,esp32s2-hal/default,esp32s2-hal/embassy-time-timg0,esp32s2-hal/embassy-executor-thread,wifi,esp-now
cargo +esp build --release --example open_access_point --target xtensa-esp32s2-none-elf --features esp32s2,esp32s2-hal/default,esp32s2-hal/embassy-time-timg0,esp32s2-hal/embassy-executor-thread,wifi,esp-now
cargo +esp build --release --example test_connect --target xtensa-esp32s2-none-elf --features esp32s2,esp32s2-hal/default,esp32s2-hal/embassy-time-timg0,esp32s2-hal/embassy-executor-thread,wifi,esp-now
copy ..\target\xtensa-esp32s2-none-elf\release\examples\esp_now_broadcaster ..\tmp\esp32s2
copy ..\target\xtensa-esp32s2-none-elf\release\examples\test_esp_now ..\tmp\esp32s2
copy ..\target\xtensa-esp32s2-none-elf\release\examples\open_access_point ..\tmp\esp32s2
copy ..\target\xtensa-esp32s2-none-elf\release\examples\test_connect ..\tmp\esp32s2

cargo +esp build --release --example esp_now_broadcaster --target xtensa-esp32s3-none-elf --features esp32s3,esp32s3-hal/default,esp32s3-hal/embassy-time-timg0,esp32s3-hal/embassy-executor-thread,wifi,esp-now
cargo +esp build --release --example test_esp_now --target xtensa-esp32s3-none-elf --features esp32s3,esp32s3-hal/default,esp32s3-hal/embassy-time-timg0,esp32s3-hal/embassy-executor-thread,wifi,esp-now
cargo +esp build --release --example open_access_point --target xtensa-esp32s3-none-elf --features esp32s3,esp32s3-hal/default,esp32s3-hal/embassy-time-timg0,esp32s3-hal/embassy-executor-thread,wifi,esp-now
cargo +esp build --release --example test_connect --target xtensa-esp32s3-none-elf --features esp32s3,esp32s3-hal/default,esp32s3-hal/embassy-time-timg0,esp32s3-hal/embassy-executor-thread,wifi,esp-now
cargo +esp build --release --example test_ble --target xtensa-esp32s3-none-elf --features esp32s3,esp32s3-hal/default,esp32s3-hal/embassy-time-timg0,esp32s3-hal/embassy-executor-thread,ble
copy ..\target\xtensa-esp32s3-none-elf\release\examples\esp_now_broadcaster ..\tmp\esp32s3
copy ..\target\xtensa-esp32s3-none-elf\release\examples\test_esp_now ..\tmp\esp32s3
copy ..\target\xtensa-esp32s3-none-elf\release\examples\open_access_point ..\tmp\esp32s3
copy ..\target\xtensa-esp32s3-none-elf\release\examples\test_connect ..\tmp\esp32s3
copy ..\target\xtensa-esp32s3-none-elf\release\examples\test_ble ..\tmp\esp32s3

cargo +nightly build --release --example esp_now_broadcaster --target riscv32imc-unknown-none-elf --features esp32c2,esp32c2-hal/default,esp32c2-hal/embassy-time-timg0,wifi,esp-now
cargo +nightly build --release --example test_esp_now --target riscv32imc-unknown-none-elf --features esp32c2,esp32c2-hal/default,esp32c2-hal/embassy-time-timg0,wifi,esp-now
cargo +nightly build --release --example open_access_point --target riscv32imc-unknown-none-elf --features esp32c2,esp32c2-hal/default,esp32c2-hal/embassy-time-timg0,wifi,esp-now
cargo +nightly build --release --example test_connect --target riscv32imc-unknown-none-elf --features esp32c2,esp32c2-hal/default,esp32c2-hal/embassy-time-timg0,wifi,esp-now
cargo +nightly build --release --example test_ble --target riscv32imc-unknown-none-elf --features esp32c2,esp32c2-hal/default,esp32c2-hal/embassy-time-timg0,ble
copy ..\target\riscv32imc-unknown-none-elf\release\examples\esp_now_broadcaster ..\tmp\esp32c2
copy ..\target\riscv32imc-unknown-none-elf\release\examples\test_esp_now ..\tmp\esp32c2
copy ..\target\riscv32imc-unknown-none-elf\release\examples\open_access_point ..\tmp\esp32c2
copy ..\target\riscv32imc-unknown-none-elf\release\examples\test_connect ..\tmp\esp32c2
copy ..\target\riscv32imc-unknown-none-elf\release\examples\test_ble ..\tmp\esp32c2

cargo +nightly build --release --example esp_now_broadcaster --target riscv32imc-unknown-none-elf --features esp32c3,esp32c3-hal/default,esp32c3-hal/embassy-time-timg0,wifi,esp-now
cargo +nightly build --release --example test_esp_now --target riscv32imc-unknown-none-elf --features esp32c3,esp32c3-hal/default,esp32c3-hal/embassy-time-timg0,wifi,esp-now
cargo +nightly build --release --example open_access_point --target riscv32imc-unknown-none-elf --features esp32c3,esp32c3-hal/default,esp32c3-hal/embassy-time-timg0,wifi,esp-now
cargo +nightly build --release --example test_connect --target riscv32imc-unknown-none-elf --features esp32c3,esp32c3-hal/default,esp32c3-hal/embassy-time-timg0,wifi,esp-now
cargo +nightly build --release --example test_ble --target riscv32imc-unknown-none-elf --features esp32c3,esp32c3-hal/default,esp32c3-hal/embassy-time-timg0,ble
copy ..\target\riscv32imc-unknown-none-elf\release\examples\esp_now_broadcaster ..\tmp\esp32c3
copy ..\target\riscv32imc-unknown-none-elf\release\examples\test_esp_now ..\tmp\esp32c3
copy ..\target\riscv32imc-unknown-none-elf\release\examples\open_access_point ..\tmp\esp32c3
copy ..\target\riscv32imc-unknown-none-elf\release\examples\test_connect ..\tmp\esp32c3
copy ..\target\riscv32imc-unknown-none-elf\release\examples\test_ble ..\tmp\esp32c3

cargo +nightly build --release --example esp_now_broadcaster --target riscv32imac-unknown-none-elf --features esp32c6,esp32c6-hal/default,esp32c6-hal/embassy-time-timg0,wifi,esp-now
cargo +nightly build --release --example test_esp_now --target riscv32imac-unknown-none-elf --features esp32c6,esp32c6-hal/default,esp32c6-hal/embassy-time-timg0,wifi,esp-now
cargo +nightly build --release --example open_access_point --target riscv32imac-unknown-none-elf --features esp32c6,esp32c6-hal/default,esp32c6-hal/embassy-time-timg0,wifi,esp-now
cargo +nightly build --release --example test_connect --target riscv32imac-unknown-none-elf --features esp32c6,esp32c6-hal/default,esp32c6-hal/embassy-time-timg0,wifi,esp-now
cargo +nightly build --release --example test_ble --target riscv32imac-unknown-none-elf --features esp32c6,esp32c6-hal/default,esp32c6-hal/embassy-time-timg0,ble
copy ..\target\riscv32imac-unknown-none-elf\release\examples\esp_now_broadcaster ..\tmp\esp32c6
copy ..\target\riscv32imac-unknown-none-elf\release\examples\test_esp_now ..\tmp\esp32c6
copy ..\target\riscv32imac-unknown-none-elf\release\examples\open_access_point ..\tmp\esp32c6
copy ..\target\riscv32imac-unknown-none-elf\release\examples\test_connect ..\tmp\esp32c6
copy ..\target\riscv32imac-unknown-none-elf\release\examples\test_ble ..\tmp\esp32c6

cargo +nightly build --release --example test_ble --target riscv32imac-unknown-none-elf --no-default-features --features esp32h2,esp32h2-hal/default,esp32h2-hal/embassy-time-timg0,ble
copy ..\target\riscv32imac-unknown-none-elf\release\examples\test_ble ..\tmp\esp32h2

cd ..\tmp
echo "Connect ESP32, ESP32-C2, ESP32-C3, ESP32-C6, ESP32-H2"
pause
esp-testrun --esp32=esp32 --esp32c2=esp32c2 --esp32c3=esp32c3 --esp32c6=esp32c6

echo "Connect ESP32, ESP32-S2, ESP32-S3, ESP32-C3"
pause
esp-testrun --esp32=esp32 --esp32s2=esp32s2 --esp32s3=esp32s3 --esp32c3=esp32c3

cd ..
rd /q /s tmp
