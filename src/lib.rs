#![no_std]
#![allow(clippy::missing_safety_doc)]
#![cfg_attr(feature = "intrinsics", feature(abi_custom))]
pub use lpc11uxx2 as pac;

mod fmt;
mod intrinsics;

pub mod clocks;
pub mod rom;
pub mod gpio;
pub mod adc;
pub mod ct;
pub mod eeprom;
pub mod flash;
pub mod i2c;
pub mod ssp;
pub mod usart;
pub mod usb;
pub mod watchdog;

embassy_hal_internal::interrupt_mod! {
    PIN_INT0,
    PIN_INT1,
    PIN_INT2,
    PIN_INT3,
    PIN_INT4,
    PIN_INT5,
    PIN_INT6,
    PIN_INT7,
    GINT0,
    GINT1,
    SSP1,
    I2C,
    CT16B0,
    CT16B1,
    CT32B0,
    CT32B1,
    SSP0,
    USART,
    USB_IRQ,
    USB_FIQ,
    ADC,
    WDT,
    BOD_IRQ,
    FLASH_IRQ,
    USBWAKEUP,
}

embassy_hal_internal::peripherals! {
    PIO0_0,
    PIO0_1,
    PIO0_2,
    PIO0_3,
    PIO0_4,
    PIO0_5,
    PIO0_6,
    PIO0_7,
    PIO0_8,
    PIO0_9,
    PIO0_10,
    PIO0_11,
    PIO0_12,
    PIO0_13,
    PIO0_14,
    PIO0_15,
    PIO0_16,
    PIO0_17,
    PIO0_18,
    PIO0_19,
    PIO0_20,
    PIO0_21,
    PIO0_22,
    PIO0_23,

    #[cfg(feature = "lqfp64")]
    PIO1_0,
    #[cfg(feature = "lqfp64")]
    PIO1_1,
    #[cfg(feature = "lqfp64")]
    PIO1_2,
    #[cfg(feature = "lqfp64")]
    PIO1_3,
    #[cfg(feature = "lqfp64")]
    PIO1_4,
    #[cfg(any(feature = "lqfp64", feature = "tfbga48"))]
    PIO1_5,
    #[cfg(feature = "lqfp64")]
    PIO1_6,
    #[cfg(feature = "lqfp64")]
    PIO1_7,
    #[cfg(feature = "lqfp64")]
    PIO1_8,
    #[cfg(feature = "lqfp64")]
    PIO1_9,
    #[cfg(feature = "lqfp64")]
    PIO1_10,
    #[cfg(feature = "lqfp64")]
    PIO1_11,
    #[cfg(feature = "lqfp64")]
    PIO1_12,
    #[cfg(any(feature = "lqfp64", feature = "lqfp48", feature = "tfbga48"))]
    PIO1_13,
    #[cfg(any(feature = "lqfp64", feature = "lqfp48", feature = "tfbga48"))]
    PIO1_14,
    PIO1_15,
    #[cfg(any(feature = "lqfp64", feature = "lqfp48", feature = "tfbga48"))]
    PIO1_16,
    #[cfg(feature = "lqfp64")]
    PIO1_17,
    #[cfg(feature = "lqfp64")]
    PIO1_18,
    PIO1_19,
    #[cfg(any(feature = "lqfp64", feature = "lqfp48", feature = "tfbga48"))]
    PIO1_20,
    #[cfg(any(feature = "lqfp64", feature = "lqfp48", feature = "tfbga48"))]
    PIO1_21,
    #[cfg(any(feature = "lqfp64", feature = "lqfp48", feature = "tfbga48"))]
    PIO1_22,
    #[cfg(any(feature = "lqfp64", feature = "lqfp48", feature = "tfbga48"))]
    PIO1_23,
    #[cfg(any(feature = "lqfp64", feature = "lqfp48", feature = "tfbga48"))]
    PIO1_24,
    #[cfg(any(feature = "lqfp64", feature = "lqfp48", feature = "tfbga48"))]
    PIO1_25,
    #[cfg(any(feature = "lqfp64", feature = "lqfp48", feature = "tfbga48"))]
    PIO1_26,
    #[cfg(any(feature = "lqfp64", feature = "lqfp48", feature = "tfbga48"))]
    PIO1_27,
    #[cfg(any(feature = "lqfp64", feature = "lqfp48", feature = "tfbga48"))]
    PIO1_28,
    #[cfg(any(feature = "lqfp64", feature = "lqfp48", feature = "tfbga48"))]
    PIO1_29,
    #[cfg(feature = "lqfp48")]
    PIO1_31,

    USB,
    I2C,
    SSP0,
    SSP1,
    CT16B0,
    CT16B1,
    CT32B0,
    CT32B1,
    USART,
    WWDT,
    ADC,
    EEPROM,
    FLASH,
}

pub mod config {
    #[non_exhaustive]
    pub struct Config {

    }

    impl Default for Config {
        fn default() -> Self {
            todo!()
        }
    }

    impl Config {
        pub fn new() -> Self {
            todo!()
        }
    }
}

pub fn init(_config: config::Config) -> Peripherals {
    let _peripherals = Peripherals::take();

    todo!()
}
