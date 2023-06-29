/**
 * ROM APIs
 */

PROVIDE ( esp_rom_crc32_le = crc32_le );
PROVIDE ( esp_rom_crc16_le = crc16_le );
PROVIDE ( esp_rom_crc8_le  = crc8_le );

PROVIDE ( esp_rom_gpio_pad_select_gpio    = gpio_pad_select_gpio );
PROVIDE ( esp_rom_gpio_pad_pullup_only    = gpio_pad_pullup );
PROVIDE ( esp_rom_gpio_pad_set_drv        = gpio_pad_set_drv );
PROVIDE ( esp_rom_gpio_pad_unhold         = gpio_pad_unhold );
PROVIDE ( esp_rom_gpio_connect_in_signal  = gpio_matrix_in );
PROVIDE ( esp_rom_gpio_connect_out_signal = gpio_matrix_out );

PROVIDE ( esp_rom_efuse_mac_address_crc8       = esp_crc8 );
PROVIDE ( esp_rom_efuse_get_flash_gpio_info    = ets_efuse_get_spiconfig );
PROVIDE ( esp_rom_efuse_get_flash_wp_gpio      = ets_efuse_get_wp_pad );
PROVIDE ( esp_rom_efuse_get_opiconfig          = ets_efuse_get_opiconfig );
PROVIDE ( esp_rom_efuse_is_secure_boot_enabled = ets_efuse_secure_boot_enabled );

PROVIDE ( esp_rom_uart_flush_tx       = uart_tx_flush );
PROVIDE ( esp_rom_uart_tx_one_char    = uart_tx_one_char );
PROVIDE ( esp_rom_uart_tx_wait_idle   = uart_tx_wait_idle );
PROVIDE ( esp_rom_uart_rx_one_char    = uart_rx_one_char );
PROVIDE ( esp_rom_uart_rx_string      = UartRxString );
PROVIDE ( esp_rom_uart_set_as_console = uart_tx_switch );
PROVIDE ( esp_rom_uart_usb_acm_init   = Uart_Init_USB );
PROVIDE ( esp_rom_uart_putc           = ets_write_char_uart );

/* wpa_supplicant re-implements the MD5 functions: MD5Init, MD5Update, MD5Final */
/* so here we directly assign the symbols with the ROM API address */
PROVIDE ( esp_rom_md5_init   = 0x4000526c );
PROVIDE ( esp_rom_md5_update = 0x4000528c );
PROVIDE ( esp_rom_md5_final  = 0x4000530c );

PROVIDE ( esp_rom_printf   = ets_printf );
PROVIDE ( esp_rom_delay_us = ets_delay_us );
PROVIDE ( esp_rom_install_uart_printf = ets_install_uart_printf );
PROVIDE ( esp_rom_get_reset_reason = rtc_get_reset_reason );
PROVIDE ( esp_rom_route_intr_matrix = intr_matrix_set );
PROVIDE ( esp_rom_get_cpu_ticks_per_us = ets_get_cpu_frequency );

PROVIDE ( esp_rom_spiflash_clear_bp = esp_rom_spiflash_unlock );
PROVIDE ( esp_rom_spiflash_write_enable = SPI_write_enable);

PROVIDE ( esp_rom_spiflash_fix_dummylen = spi_dummy_len_fix );
PROVIDE ( esp_rom_spiflash_set_drvs = SetSpiDrvs);
PROVIDE ( esp_rom_spiflash_select_padsfunc = SelectSpiFunction );
PROVIDE ( esp_rom_spiflash_common_cmd = SPI_Common_Command );

/* from esp32s2.rom.ld */
PROVIDE ( ets_delay_us = 0x4000d888 );
PROVIDE ( roundup2 = 0x4001bcd0 );
PROVIDE ( g_phyFuns = 0x3ffffd90 );
PROVIDE ( g_phyFuns_instance = 0x3ffffd94 );
PROVIDE ( phy_get_romfuncs = 0x4000a88c );
PROVIDE ( rom_read_hw_noisefloor = 0x40009c38 );
PROVIDE ( rom_phy_disable_low_rate = 0x4000a2b8 );
PROVIDE ( rom_phy_enable_low_rate = 0x4000a280 );
PROVIDE ( intr_matrix_set = 0x4000f1d0 );

/* from esp32s2.rom.newlib-funcs.ld */
strncpy = 0x40007f20;
strcpy = 0x40007cfc;
strncmp = 0x4001ae64;

/* from esp32s3.rom.libgcc.ld*/
__popcountsi2 = 0x40008fa8;
__bswapsi2 = 0x40006d0c;

/* from esp32s3.rom.newlib.ld */
strcmp = 0x40007be4;
strstr = 0x4001aee8;
strchr = 0x4001adb0;

PROVIDE ( crc32_le = 0x400119dc );
