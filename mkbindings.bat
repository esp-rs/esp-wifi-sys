set OPTS=--no-derive-debug --raw-line "#![allow(non_camel_case_types,non_snake_case,non_upper_case_globals,dead_code)]" --use-core --ctypes-prefix "crate::binary::c_types" --no-layout-tests 

set SYSROOT=%HOMEPATH%/.espressif/tools/riscv32-esp-elf/esp-2021r2-8.4.0/riscv32-esp-elf/"
set INCL=%HOMEPATH%\.espressif\tools\riscv32-esp-elf\esp-2021r2-8.4.0\riscv32-esp-elf\riscv32-esp-elf\include\
bindgen %OPTS% include\include.h > src\binary\include_esp32c3.rs -- -I./headers/ -I%INCL% -I./include/ -DCONFIG_IDF_TARGET_ESP32C3 -I./headers/esp32c3/ --sysroot=%SYSROOT% --target=riscv32

set SYSROOT=%HOMEPATH%/.espressif\tools\xtensa-esp32-elf\esp-2021r2-8.4.0\xtensa-esp32-elf/"
set INCL=%HOMEPATH%\.espressif\tools\xtensa-esp32-elf\esp-2021r2-8.4.0\xtensa-esp32-elf\xtensa-esp32-elf\include
bindgen %OPTS% include\include.h > src\binary\include_esp32.rs -- -I./headers/ -I%INCL% -I./include/ -DCONFIG_IDF_TARGET_ESP32 -I./headers/esp32/ --sysroot=%SYSROOT% --target=xtensa

set SYSROOT=%HOMEPATH%/.espressif\tools\xtensa-esp32s3-elf\esp-2021r2-8.4.0\xtensa-esp32s3-elf/"
set INCL=%HOMEPATH%/.espressif\tools\xtensa-esp32s3-elf\esp-2021r2-8.4.0\xtensa-esp32s3-elf\xtensa-esp32s3-elf\include
bindgen %OPTS% include\include.h > src\binary\include_esp32s3.rs -- -I./headers/ -I%INCL% -I./include/ -DCONFIG_IDF_TARGET_ESP32S3 -I./headers/esp32s3/ --sysroot=%SYSROOT% --target=xtensa

set SYSROOT=%HOMEPATH%/.espressif\tools\xtensa-esp32s2-elf\esp-2021r2-8.4.0\xtensa-esp32s2-elf/"
set INCL=%HOMEPATH%/.espressif\tools\xtensa-esp32s2-elf\esp-2021r2-8.4.0\xtensa-esp32s2-elf\xtensa-esp32s2-elf\include
bindgen %OPTS% include\include.h > src\binary\include_esp32s2.rs -- -I./headers/ -I%INCL% -I./include/ -DCONFIG_IDF_TARGET_ESP32S2 -I./headers/esp32s2/ --sysroot=%SYSROOT% --target=xtensa
