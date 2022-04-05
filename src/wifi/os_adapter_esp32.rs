#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(non_snake_case)]
use crate::binary::include::*;
use crate::trace;
use crate::wifi::phy_init_data_esp32::PHY_INIT_DATA_DEFAULT;

pub(crate) fn chip_ints_on(mask: u32) {
    trace!("chip_ints_on esp32");
    unsafe {
        xtensa_lx::interrupt::enable_mask(1 << 0);
    }
}

pub(crate) fn init_clocks() {
    unsafe {
        let mut regval = getreg32(0x6001f048); /* DR_REG_BB_BASE+48 */
        regval &= !(1 << 14);
        putreg32(regval, 0x6001f048);
    }

    let xtal_freq = 40; // RTC_XTAL_FREQ_40M;
    let source_freq_mhz = 480; // RTC_PLL_FREQ_480M;

    //esp32_rtc_update_to_xtal(xtal_freq, 1);
    esp32_rtc_bbpll_enable();
    esp32_rtc_bbpll_configure(xtal_freq, source_freq_mhz);
    set_cpu_freq(240);

    // make debug output show up again
    extern "C" {
        fn uart_div_modify(uart: u32, value: u32);
    }
    unsafe {
        const UART_CLOCK_MHZ: u32 = 40;
        uart_div_modify(0, ((UART_CLOCK_MHZ * 1_000_000) << 4) / 115200);
        for _ in 0..20_000 {}
    }
}

const RTC_CNTL_DBIAS_1V10: u32 = 4;
const RTC_CNTL_DBIAS_1V00: u32 = 2;
const DIG_DBIAS_80M_160M: u32 = RTC_CNTL_DBIAS_1V10;

const RTC_CNTL_DBIAS_1V25: u32 = 7;

const DR_REG_EFUSE_BASE: u32 = 0x3ff5a000;
const EFUSE_BLK0_RDATA5_REG: u32 = DR_REG_EFUSE_BASE + 0x014;

const EFUSE_RD_VOL_LEVEL_HP_INV_V: u32 = 0x03;
const EFUSE_RD_VOL_LEVEL_HP_INV_S: u32 = 22;

const CPU_80M: u32 = 0;
const CPU_160M: u32 = 1;
const CPU_240M: u32 = 2;
const MHZ: u32 = 1000000;
const UINT16_MAX: u32 = 0xffff;
const DR_REG_DPORT_BASE: u32 = 0x3ff00000;
const DR_REG_RTCCNTL_BASE: u32 = 0x3ff48000;
const DPORT_CPU_PER_CONF_REG: u32 = DR_REG_DPORT_BASE + 0x03C;
const RTC_CNTL_REG: u32 = DR_REG_RTCCNTL_BASE + 0x7c;

const RTC_CNTL_DIG_DBIAS_WAK: u32 = 0x00000007;
const RTC_CNTL_DIG_DBIAS_WAK_V: u32 = 0x7;
const RTC_CNTL_DIG_DBIAS_WAK_S: u32 = 11;

const RTC_CNTL_CLK_CONF_REG: u32 = DR_REG_RTCCNTL_BASE + 0x70;

const RTC_CNTL_SOC_CLK_SEL_V: u32 = 0x3;
const RTC_CNTL_SOC_CLK_SEL_S: u32 = 27;

const RTC_CNTL_SOC_CLK_SEL_PLL: u32 = 1;

const RTC_APB_FREQ_REG: u32 = RTC_CNTL_STORE5_REG;
const RTC_CNTL_STORE5_REG: u32 = DR_REG_RTCCNTL_BASE + 0xb4;

const DPORT_WIFI_CLK_EN_REG: u32 = DR_REG_DPORT_BASE + 0x0CC;
const DPORT_WIFI_CLK_WIFI_EN: u32 = 0x00000406;
const DPORT_WIFI_CLK_WIFI_EN_V: u32 = 0x406;
const DPORT_WIFI_CLK_WIFI_EN_S: u32 = 0;
const DPORT_WIFI_CLK_WIFI_EN_M: u32 = (DPORT_WIFI_CLK_WIFI_EN_V) << (DPORT_WIFI_CLK_WIFI_EN_S);

const DR_REG_APB_CTRL_BASE: u32 = 0x3ff66000;
const APB_CTRL_SYSCLK_CONF_REG: u32 = DR_REG_APB_CTRL_BASE + 0x0;

// Mask for clock bits used by both WIFI and Bluetooth
const DPORT_WIFI_CLK_WIFI_BT_COMMON_M: u32 = 0x000003c9;

const APB_CTRL_PRE_DIV_CNT_V: u32 = 0x3ff;
const APB_CTRL_PRE_DIV_CNT_S: u32 = 0;

const APB_CTRL_XTAL_TICK_CONF_REG: u32 = DR_REG_APB_CTRL_BASE + 0x4;

const REF_CLK_FREQ: u32 = 1000000;

const RTC_CNTL_SOC_CLK_SEL_XTL: u32 = 0;

const DIG_DBIAS_2M: u32 = RTC_CNTL_DBIAS_1V00;
const DIG_DBIAS_XTAL: u32 = RTC_CNTL_DBIAS_1V10;

const RTC_CNTL_OPTIONS0_REG: u32 = DR_REG_RTCCNTL_BASE + 0x0;
const RTC_CNTL_BIAS_I2C_FORCE_PD: u32 = 1 << 18;
const RTC_CNTL_BB_I2C_FORCE_PD: u32 = 1 << 6;
const RTC_CNTL_BBPLL_FORCE_PD: u32 = 1 << 10;
const RTC_CNTL_BBPLL_I2C_FORCE_PD: u32 = 1 << 8;

const I2C_BBPLL: u32 = 0x66;
const I2C_BBPLL_HOSTID: u32 = 4;

const I2C_BBPLL_IR_CAL_DELAY: u32 = 0;
const I2C_BBPLL_IR_CAL_EXT_CAP: u32 = 1;
const I2C_BBPLL_OC_ENB_FCAL: u32 = 4;
const I2C_BBPLL_OC_ENB_VCON: u32 = 10;
const I2C_BBPLL_BBADC_CAL_7_0: u32 = 12;

const BBPLL_IR_CAL_DELAY_VAL: u32 = 0x18;
const BBPLL_IR_CAL_EXT_CAP_VAL: u32 = 0x20;
const BBPLL_OC_ENB_FCAL_VAL: u32 = 0x9a;
const BBPLL_OC_ENB_VCON_VAL: u32 = 0x00;
const BBPLL_BBADC_CAL_7_0_VAL: u32 = 0x00;

const RTC_PLL_FREQ_320M: u32 = 320;
const RTC_PLL_FREQ_480M: u32 = 480;

const I2C_BBPLL_ENDIV5: u32 = 11;

const BBPLL_ENDIV5_VAL_320M: u32 = 0x43;
const BBPLL_BBADC_DSMP_VAL_320M: u32 = 0x84;
const BBPLL_ENDIV5_VAL_480M: u32 = 0xc3;
const BBPLL_BBADC_DSMP_VAL_480M: u32 = 0x74;

const I2C_BBPLL_BBADC_DSMP: u32 = 9;
const I2C_BBPLL_OC_LREF: u32 = 2;
const I2C_BBPLL_OC_DIV_7_0: u32 = 3;
const I2C_BBPLL_OC_DCUR: u32 = 5;

#[allow(unreachable_patterns)]
fn esp32_rtc_bbpll_configure(xtal_freq: u32, pll_freq: u32) {
    unsafe {
        let RTC_CNTL_DBIAS_HP_VOLT: u32 = RTC_CNTL_DBIAS_1V25
            - (reg_get_field(
                EFUSE_BLK0_RDATA5_REG,
                EFUSE_RD_VOL_LEVEL_HP_INV_S,
                EFUSE_RD_VOL_LEVEL_HP_INV_V,
            ));
        let DIG_DBIAS_240M: u32 = RTC_CNTL_DBIAS_HP_VOLT;

        let div_ref: u32;
        let div7_0: u32;
        let div10_8: u32;
        let lref: u32;
        let dcur: u32;
        let bw: u32;
        let i2c_bbpll_lref: u32;
        let i2c_bbpll_div_7_0: u32;
        let i2c_bbpll_dcur: u32;

        if pll_freq == RTC_PLL_FREQ_320M {
            /* Raise the voltage, if needed */

            // REG_SET_FIELD(
            //     RTC_CNTL_REG,
            //     RTC_CNTL_DIG_DBIAS_WAK_S,
            //     RTC_CNTL_DIG_DBIAS_WAK_V,
            //     DIG_DBIAS_80M_160M,
            // );

            /* Configure 320M PLL */
            match xtal_freq {
                RTC_XTAL_FREQ_40M => {
                    div_ref = 0;
                    div7_0 = 32;
                    div10_8 = 0;
                    lref = 0;
                    dcur = 6;
                    bw = 3;
                }

                RTC_XTAL_FREQ_26M => {
                    div_ref = 12;
                    div7_0 = 224;
                    div10_8 = 4;
                    lref = 1;
                    dcur = 0;
                    bw = 1;
                }

                RTC_XTAL_FREQ_24M => {
                    div_ref = 11;
                    div7_0 = 224;
                    div10_8 = 4;
                    lref = 1;
                    dcur = 0;
                    bw = 1;
                }

                _ => {
                    div_ref = 12;
                    div7_0 = 224;
                    div10_8 = 4;
                    lref = 0;
                    dcur = 0;
                    bw = 0;
                }
            }

            I2C_WRITEREG_RTC(
                I2C_BBPLL,
                I2C_BBPLL_HOSTID,
                I2C_BBPLL_ENDIV5,
                BBPLL_ENDIV5_VAL_320M,
            );
            I2C_WRITEREG_RTC(
                I2C_BBPLL,
                I2C_BBPLL_HOSTID,
                I2C_BBPLL_BBADC_DSMP,
                BBPLL_BBADC_DSMP_VAL_320M,
            );
        } else {
            /* Raise the voltage */

            // REG_SET_FIELD(
            //     RTC_CNTL_REG,
            //     RTC_CNTL_DIG_DBIAS_WAK_S,
            //     RTC_CNTL_DIG_DBIAS_WAK_V,
            //     DIG_DBIAS_240M,
            // );
            //ets_delay_us(DELAY_PLL_DBIAS_RAISE);
            for _ in 0..100_000 {}

            /* Configure 480M PLL */

            match xtal_freq {
                RTC_XTAL_FREQ_40M => {
                    div_ref = 0;
                    div7_0 = 28;
                    div10_8 = 0;
                    lref = 0;
                    dcur = 6;
                    bw = 3;
                }

                RTC_XTAL_FREQ_26M => {
                    div_ref = 12;
                    div7_0 = 144;
                    div10_8 = 4;
                    lref = 1;
                    dcur = 0;
                    bw = 1;
                }

                RTC_XTAL_FREQ_24M => {
                    div_ref = 11;
                    div7_0 = 144;
                    div10_8 = 4;
                    lref = 1;
                    dcur = 0;
                    bw = 1;
                }

                _ => {
                    div_ref = 12;
                    div7_0 = 224;
                    div10_8 = 4;
                    lref = 0;
                    dcur = 0;
                    bw = 0;
                }
            }

            I2C_WRITEREG_RTC(
                I2C_BBPLL,
                I2C_BBPLL_HOSTID,
                I2C_BBPLL_ENDIV5,
                BBPLL_ENDIV5_VAL_480M,
            );

            I2C_WRITEREG_RTC(
                I2C_BBPLL,
                I2C_BBPLL_HOSTID,
                I2C_BBPLL_BBADC_DSMP,
                BBPLL_BBADC_DSMP_VAL_480M,
            );
        }

        i2c_bbpll_lref = (lref << 7) | (div10_8 << 4) | (div_ref);
        i2c_bbpll_div_7_0 = div7_0;
        i2c_bbpll_dcur = (bw << 6) | dcur;
        I2C_WRITEREG_RTC(
            I2C_BBPLL,
            I2C_BBPLL_HOSTID,
            I2C_BBPLL_OC_LREF,
            i2c_bbpll_lref,
        );

        I2C_WRITEREG_RTC(
            I2C_BBPLL,
            I2C_BBPLL_HOSTID,
            I2C_BBPLL_OC_DIV_7_0,
            i2c_bbpll_div_7_0,
        );

        I2C_WRITEREG_RTC(
            I2C_BBPLL,
            I2C_BBPLL_HOSTID,
            I2C_BBPLL_OC_DCUR,
            i2c_bbpll_dcur,
        );
    }
}

fn esp32_rtc_bbpll_enable() {
    unsafe {
        // modifyreg32(
        //     RTC_CNTL_OPTIONS0_REG,
        //     RTC_CNTL_BIAS_I2C_FORCE_PD
        //         | RTC_CNTL_BB_I2C_FORCE_PD
        //         | RTC_CNTL_BBPLL_FORCE_PD
        //         | RTC_CNTL_BBPLL_I2C_FORCE_PD,
        //     0,
        // );

        /* reset BBPLL configuration */
        I2C_WRITEREG_RTC(
            I2C_BBPLL,
            I2C_BBPLL_HOSTID,
            I2C_BBPLL_IR_CAL_DELAY,
            BBPLL_IR_CAL_DELAY_VAL,
        );
        I2C_WRITEREG_RTC(
            I2C_BBPLL,
            I2C_BBPLL_HOSTID,
            I2C_BBPLL_IR_CAL_EXT_CAP,
            BBPLL_IR_CAL_EXT_CAP_VAL,
        );
        I2C_WRITEREG_RTC(
            I2C_BBPLL,
            I2C_BBPLL_HOSTID,
            I2C_BBPLL_OC_ENB_FCAL,
            BBPLL_OC_ENB_FCAL_VAL,
        );
        I2C_WRITEREG_RTC(
            I2C_BBPLL,
            I2C_BBPLL_HOSTID,
            I2C_BBPLL_OC_ENB_VCON,
            BBPLL_OC_ENB_VCON_VAL,
        );
        I2C_WRITEREG_RTC(
            I2C_BBPLL,
            I2C_BBPLL_HOSTID,
            I2C_BBPLL_BBADC_CAL_7_0,
            BBPLL_BBADC_CAL_7_0_VAL,
        );
    }
}

#[inline(always)]
unsafe fn I2C_WRITEREG_RTC(block: u32, block_hostid: u32, reg_add: u32, indata: u32) {
    extern "C" {
        pub fn rom_i2c_writeReg(block: u32, block_hostid: u32, reg_add: u32, indata: u32) -> i32;
    }

    rom_i2c_writeReg(block, block_hostid, reg_add, indata);
}

fn esp32_rtc_update_to_xtal(freq: u32, div: u32) {
    unsafe {
        let value =
            (((freq * MHZ) >> 12) & UINT16_MAX) | ((((freq * MHZ) >> 12) & UINT16_MAX) << 16);
        // esp32_update_cpu_freq(freq); // just for ets_delay_us
        /* set divider from XTAL to APB clock */
        reg_set_field(
            APB_CTRL_SYSCLK_CONF_REG,
            APB_CTRL_PRE_DIV_CNT_S,
            APB_CTRL_PRE_DIV_CNT_V,
            div - 1,
        );

        /* adjust ref_tick */
        modifyreg32(
            APB_CTRL_XTAL_TICK_CONF_REG,
            0,
            (freq * MHZ) / REF_CLK_FREQ - 1,
        );

        /* switch clock source */

        reg_set_field(
            RTC_CNTL_CLK_CONF_REG,
            RTC_CNTL_SOC_CLK_SEL_S,
            RTC_CNTL_SOC_CLK_SEL_V,
            RTC_CNTL_SOC_CLK_SEL_XTL,
        );

        putreg32(value, RTC_APB_FREQ_REG);

        /* lower the voltage */

        if freq <= 2 {
            reg_set_field(
                RTC_CNTL_REG,
                RTC_CNTL_DIG_DBIAS_WAK_S,
                RTC_CNTL_DIG_DBIAS_WAK_V,
                DIG_DBIAS_2M,
            );
        } else {
            reg_set_field(
                RTC_CNTL_REG,
                RTC_CNTL_DIG_DBIAS_WAK_S,
                RTC_CNTL_DIG_DBIAS_WAK_V,
                DIG_DBIAS_XTAL,
            );
        }
    }
}

fn set_cpu_freq(cpu_freq_mhz: u32) {
    unsafe {
        const RTC_CNTL_DBIAS_1V10: u32 = 4;
        const DIG_DBIAS_80M_160M: u32 = RTC_CNTL_DBIAS_1V10;

        const RTC_CNTL_DBIAS_1V25: u32 = 7;

        const DR_REG_EFUSE_BASE: u32 = 0x3ff5a000;
        const EFUSE_BLK0_RDATA5_REG: u32 = DR_REG_EFUSE_BASE + 0x014;

        const EFUSE_RD_VOL_LEVEL_HP_INV_V: u32 = 0x03;
        const EFUSE_RD_VOL_LEVEL_HP_INV_S: u32 = 22;

        let RTC_CNTL_DBIAS_HP_VOLT: u32 = RTC_CNTL_DBIAS_1V25
            - (reg_get_field(
                EFUSE_BLK0_RDATA5_REG,
                EFUSE_RD_VOL_LEVEL_HP_INV_S,
                EFUSE_RD_VOL_LEVEL_HP_INV_V,
            ));
        let DIG_DBIAS_240M: u32 = RTC_CNTL_DBIAS_HP_VOLT;

        const CPU_80M: u32 = 0;
        const CPU_160M: u32 = 1;
        const CPU_240M: u32 = 2;
        const MHZ: u32 = 1000000;
        const UINT16_MAX: u32 = 0xffff;
        const DR_REG_DPORT_BASE: u32 = 0x3ff00000;
        const DR_REG_RTCCNTL_BASE: u32 = 0x3ff48000;
        const DPORT_CPU_PER_CONF_REG: u32 = DR_REG_DPORT_BASE + 0x03C;
        const RTC_CNTL_REG: u32 = DR_REG_RTCCNTL_BASE + 0x7c;

        const RTC_CNTL_DIG_DBIAS_WAK: u32 = 0x00000007;
        const RTC_CNTL_DIG_DBIAS_WAK_V: u32 = 0x7;
        const RTC_CNTL_DIG_DBIAS_WAK_S: u32 = 11;

        const RTC_CNTL_CLK_CONF_REG: u32 = DR_REG_RTCCNTL_BASE + 0x70;

        const RTC_CNTL_SOC_CLK_SEL_V: u32 = 0x3;
        const RTC_CNTL_SOC_CLK_SEL_S: u32 = 27;

        const RTC_CNTL_SOC_CLK_SEL_PLL: u32 = 1;

        const RTC_APB_FREQ_REG: u32 = RTC_CNTL_STORE5_REG;
        const RTC_CNTL_STORE5_REG: u32 = DR_REG_RTCCNTL_BASE + 0xb4;

        let mut dbias = DIG_DBIAS_80M_160M;
        let mut per_conf = CPU_240M;

        match cpu_freq_mhz {
            160 => {
                per_conf = CPU_160M;
            }
            240 => {
                dbias = DIG_DBIAS_240M;
                per_conf = CPU_240M;
            }
            80 => {
                per_conf = CPU_80M;
            }
            _ => (),
        }

        let value = (((80 * MHZ) >> 12) & UINT16_MAX) | ((((80 * MHZ) >> 12) & UINT16_MAX) << 16);
        putreg32(per_conf, DPORT_CPU_PER_CONF_REG);
        reg_set_field(
            RTC_CNTL_REG,
            RTC_CNTL_DIG_DBIAS_WAK_S,
            RTC_CNTL_DIG_DBIAS_WAK_V,
            dbias,
        );
        reg_set_field(
            RTC_CNTL_CLK_CONF_REG,
            RTC_CNTL_SOC_CLK_SEL_S,
            RTC_CNTL_SOC_CLK_SEL_V,
            RTC_CNTL_SOC_CLK_SEL_PLL,
        );
        putreg32(value, RTC_APB_FREQ_REG);

        esp32_update_cpu_freq(cpu_freq_mhz);
        // esp32_rtc_wait_for_slow_cycle();
    }
}

/****************************************************************************
 * Name:  esp32_update_cpu_freq
 *
 * Description:
 *   Set the real CPU ticks per us to the ets, so that ets_delay_us
 *   will be accurate. Call this function when CPU frequency is changed.
 *
 * Input Parameters:
 *   ticks_per_us - CPU ticks per us
 *
 * Returned Value:
 *   None
 *
 ****************************************************************************/

fn esp32_update_cpu_freq(ticks_per_us: u32) {
    extern "C" {
        static mut g_ticks_per_us_pro: u32;
    }
    unsafe {
        /* Update scale factors used by esp_rom_delay_us */
        g_ticks_per_us_pro = ticks_per_us;
        // #ifdef CONFIG_SMP
        // g_ticks_per_us_app = ticks_per_us;
        // #endif
    }
}

#[inline(always)]
unsafe fn putreg32(v: u32, r: u32) {
    //(r as *mut u32).write_volatile(v);
    *(r as *mut u32) = v;
}

#[inline(always)]
unsafe fn getreg32(r: u32) -> u32 {
    *(r as *mut u32)
}

#[inline(always)]
unsafe fn reg_set_field(r: u32, f_s: u32, f_v: u32, v: u32) {
    putreg32(
        (getreg32(r) & !(f_v.wrapping_shl(f_s))) | (v & (f_v.wrapping_shl(f_s))),
        r,
    );
}

#[inline(always)]
unsafe fn reg_get_field(r: u32, f_s: u32, f_v: u32) -> u32 {
    (getreg32(r) >> (f_s)) & (f_v)
}

#[inline(always)]
unsafe fn modifyreg32(addr: u32, clearbits: u32, setbits: u32) {
    critical_section::with(|_| {
        let mut regval = getreg32(addr);
        regval &= !clearbits;
        regval |= setbits;
        putreg32(addr, regval);
    });
}

pub(crate) unsafe extern "C" fn wifi_int_disable(
    wifi_int_mux: *mut crate::binary::c_types::c_void,
) -> u32 {
    trace!("wifi_int_disable() esp32");
    critical_section::acquire() as u32
}

pub(crate) unsafe extern "C" fn wifi_int_restore(
    wifi_int_mux: *mut crate::binary::c_types::c_void,
    tmp: u32,
) {
    trace!("wifi_int_restore() esp32");
    critical_section::release(tmp as u8)
}

pub(crate) unsafe extern "C" fn phy_common_clock_disable() {
    trace!("phy_common_clock_disable - not implemented");
}

pub(crate) unsafe extern "C" fn phy_common_clock_enable() {
    trace!("phy_common_clock_enable - not implemented");
}

pub(crate) unsafe extern "C" fn set_intr(
    cpu_no: i32,
    intr_source: u32,
    intr_num: u32,
    intr_prio: i32,
) {
    extern "C" {
        fn intr_matrix_set(cpu_no: u32, model_num: u32, intr_num: u32);
    }
    // Force to bind WiFi interrupt to CPU0
    intr_matrix_set(0, intr_source, intr_num);
}

pub(crate) unsafe extern "C" fn phy_enable() {
    trace!("phy_enable - not fully implemented");

    static mut G_IS_PHY_CALIBRATED: bool = false;

    let mut cal_data: [u8; core::mem::size_of::<esp_phy_calibration_data_t>()] =
        [0u8; core::mem::size_of::<esp_phy_calibration_data_t>()];

    //let phy_version = get_phy_version_str();
    //trace!("phy_version {}", StrBuf::from(phy_version).as_str_ref());

    critical_section::with(|_| {
        /* Update WiFi MAC time before WiFi/BT common clock is enabled */
        //phy_update_wifi_mac_time(false, g_phy_rf_en_ts);

        phy_enable_clock();
        phy_set_wifi_mode_only(true);

        if G_IS_PHY_CALIBRATED == false {
            let init_data = &PHY_INIT_DATA_DEFAULT;

            register_chipv7_phy(
                init_data,
                &mut cal_data as *mut _ as *mut crate::binary::include::esp_phy_calibration_data_t,
                esp_phy_calibration_mode_t_PHY_RF_CAL_FULL,
            );

            G_IS_PHY_CALIBRATED = true;
        } else {
            trace!("implement phy_digital_regs_load");
            phy_wakeup_init();
            //phy_digital_regs_load();
            /*
            static inline void phy_digital_regs_load(void)
            {
            if (g_phy_digital_regs_mem != NULL)
                {
                phy_dig_reg_backup(false, g_phy_digital_regs_mem);
                }
            }
            */
        }
    });
}

/****************************************************************************
 * Name: phy_update_wifi_mac_time
 *
 * Description:
 *   Update WiFi mac timer.
 *
 * Input Parameters:
 *   en_clock_stopped - Check if clock is stopped
 *   now              - time now
 *
 * Returned Value:
 *   NOne
 *
 ****************************************************************************/
pub(crate) unsafe fn phy_update_wifi_mac_time(en_clock_stopped: bool, now: i64) {
    trace!("phy_update_wifi_mac_time not implemented");
}

pub(crate) unsafe fn phy_enable_clock() {
    trace!("phy_enable_clock");

    let ptr = DPORT_WIFI_CLK_EN_REG as *mut u32;
    let old = ptr.read_volatile();
    ptr.write_volatile(old | DPORT_WIFI_CLK_WIFI_BT_COMMON_M);
}

pub(crate) unsafe extern "C" fn read_mac(
    mac: *mut u8,
    type_: u32,
) -> crate::binary::c_types::c_int {
    trace!("read_mac {:p} {}", mac, type_);

    let mut regval = [0u32; 2];
    let data = &regval as *const _ as *const u8;
    regval[0] = (0x3ff5a004 as *const u32).read_volatile();
    regval[1] = (0x3ff5a008 as *const u32).read_volatile();

    for i in 0..6 {
        mac.offset(i)
            .write_volatile(data.offset(5 - i).read_volatile());
    }

    /* ESP_MAC_WIFI_SOFTAP */
    if type_ == 1 {
        let tmp = mac.offset(0).read_volatile();
        for i in 0..64 {
            mac.offset(0).write_volatile(tmp | 0x02);
            mac.offset(0)
                .write_volatile(mac.offset(0).read_volatile() ^ (i << 2));

            if mac.offset(0).read_volatile() != tmp {
                break;
            }
        }
    }

    0
}

pub(crate) unsafe extern "C" fn wifi_clock_enable() {
    trace!("wifi_clock_enable");

    let ptr = DPORT_WIFI_CLK_EN_REG as *mut u32;
    let old = ptr.read_volatile();
    ptr.write_volatile(old | DPORT_WIFI_CLK_WIFI_EN_M);
}

pub(crate) unsafe extern "C" fn wifi_clock_disable() {
    trace!("wifi_clock_disable");

    let ptr = DPORT_WIFI_CLK_EN_REG as *mut u32;
    let old = ptr.read_volatile();
    ptr.write_volatile(old & !DPORT_WIFI_CLK_WIFI_EN_M);
}
