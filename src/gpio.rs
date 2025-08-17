use core::convert::Infallible;

use crate::{pac, peripherals};
use embassy_hal_internal::{Peri, PeripheralType, impl_peripheral};

use pac::{
    GPIO_PORT, IOCON,
    common::{RW, Reg},
    gpio::regs::{Clr, Not, Set},
    iocon::vals::{Admode, I2cmode, Mode},
};

#[repr(u8)]
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Port {
    Port0,
    Port1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum Pull {
    None,
    /// Internal pull-up
    Up,
    /// Internal pull-down
    Down,
    /// Repeater mode/bus-keeper
    Repeater,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum Level {
    Low,
    High,
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Unsupported;

pub(crate) trait SealedPin: Sized {
    fn port_pin(&self) -> u8;

    #[inline]
    fn _pin(&self) -> u8 {
        self.port_pin() & 0x1f
    }

    #[inline]
    fn _port(&self) -> Port {
        match self.port_pin() >> 5 {
            0 => Port::Port0,
            _ => Port::Port1,
        }
    }

    #[inline]
    fn iocon(&self) -> Reg<pac::iocon::regs::Pio, RW> {
        match self._port() {
            Port::Port0 => IOCON.port0().p(self._pin() as _),
            Port::Port1 => IOCON.port1().p(self._pin() as _),
        }
    }

    #[inline]
    fn gpio_port(&self) -> pac::gpio::Port {
        GPIO_PORT.port(self._port() as _)
    }

    fn pio_func(&self) -> u8 {
        // Pins that have Func(PIO) = 1
        // - PIO0_0 (RESET)
        // - PIO0_10 (SWCLK)
        // - PIO0_11 (TDI)
        // - PIO0_12 (TMS)
        // - PIO0_13 (TDO)
        // - PIO0_14 (TRST)
        // - PIO0_15 (SWDIO)
        //
        // All other pins have Func(PIO = 0)
        match self.port_pin() {
            0 | 10 | 11 | 12 | 13 | 14 | 15 => 1,

            _ => 0,
        }
    }

    #[inline]
    fn is_i2c_pin(&self) -> bool {
        // I2C pins don't support setting their mode, and also need
        // i2cmode set to STANDARD_IO to be used as GPIO
        // - PIO0_4 (SCL)
        // - PIO0_5 (SDA)
        matches!(self.port_pin(), 4 | 5)
    }

    #[inline]
    fn is_adc_pin(&self) -> bool {
        // - PIO0_11 (AD0)
        // - PIO0_12 (AD1)
        // - PIO0_13 (AD2)
        // - PIO0_14 (AD3)
        // - PIO0_15 (AD4)
        // - PIO0_16 (AD5)
        // - PIO0_22 (AD6)
        // - PIO0_23 (AD7)
        matches!(self.port_pin(), 11 | 12 | 13 | 14 | 15 | 16 | 22 | 23)
    }
}

#[allow(private_bounds)]
pub trait Pin: SealedPin + PeripheralType + Sized + Into<AnyPin> {
    #[inline]
    fn pin(&self) -> u8 {
        self._pin()
    }

    #[inline]
    fn port(&self) -> Port {
        self._port()
    }
}

pub struct AnyPin {
    port_pin: u8,
}

impl AnyPin {
    pub unsafe fn steal(port: Port, pin: u8) -> Peri<'static, Self> {
        let port_pin = (port as u8) << 5 | pin;
        unsafe { Peri::new_unchecked(Self { port_pin }) }
    }
}

impl Pin for AnyPin {}

impl SealedPin for AnyPin {
    fn port_pin(&self) -> u8 {
        self.port_pin
    }
}

impl_peripheral!(AnyPin);

impl core::fmt::Debug for AnyPin {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "AnyPin(Pio{}_{})", self.port() as u8, self.pin())
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for AnyPin {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "AnyPin(Pio{=u8}_{=u8})", self.port() as u8, self.pin())
    }
}

/// GPIO flexible pin.
///
/// This pin can be either an input or output pin. The output level register bit will remain
/// set while not in output mode, so the pin's level will be 'remembered' when it is not in output
/// mode.
///
/// The I2C pins do not support most of the configuration options:
/// - [`PIO0_4`][crate::peripherals::PIO0_4]
/// - [`PIO0_5`][crate::peripherals::PIO0_5]
///
/// [`Flex::set_glitch_filtering`] is only available on ADC pins:
/// - [`PIO0_11`][crate::peripherals::PIO0_11]
/// - [`PIO0_12`][crate::peripherals::PIO0_12]
/// - [`PIO0_13`][crate::peripherals::PIO0_13]
/// - [`PIO0_14`][crate::peripherals::PIO0_14]
/// - [`PIO0_15`][crate::peripherals::PIO0_15]
/// - [`PIO0_16`][crate::peripherals::PIO0_16]
/// - [`PIO0_22`][crate::peripherals::PIO0_22]
/// - [`PIO0_23`][crate::peripherals::PIO0_23]
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Flex<'d> {
    pin: Peri<'d, AnyPin>,
}

impl<'d> Flex<'d> {
    pub fn new(pin: Peri<'d, impl Pin + 'd>) -> Self {
        pin.iocon().modify(|r: &mut lpc11uxx2::iocon::regs::Pio| {
            r.set_func(pin.pio_func());

            if pin.is_adc_pin() {
                r.set_admode(Admode::DIGITAL);
            }

            if pin.is_i2c_pin() {
                r.set_i2cmode(I2cmode::PIO);
            }
        });
        Self { pin: pin.into() }
    }

    #[inline]
    fn pin(&self) -> u8 {
        self.pin.pin()
    }

    #[inline]
    fn bit(&self) -> u32 {
        1 << self.pin()
    }

    /// Set the pin's pull
    ///
    /// Not supported on I2C pins
    #[inline]
    pub fn set_pull(&mut self, pull: Pull) -> Result<(), Unsupported> {
        if self.pin.is_i2c_pin() && pull != Pull::None {
            return Err(Unsupported);
        }

        self.pin.iocon().modify(|r| {
            r.set_mode(match pull {
                Pull::None => Mode::FLOATING,
                Pull::Up => Mode::PULL_UP,
                Pull::Down => Mode::PULL_DOWN,
                Pull::Repeater => Mode::REPEATER_MODE,
            });
        });
        Ok(())
    }

    /// Sets the pin's hysteresis buffer
    #[inline]
    pub fn set_hysteresis(&mut self, enable: bool) -> Result<(), Unsupported> {
        if self.pin.is_i2c_pin() {
            return Err(Unsupported);
        }

        self.pin.iocon().modify(|r| r.set_hys(enable));
        Ok(())
    }

    /// Configure the input logic conversion of this pin.
    #[inline]
    pub fn set_input_inversion(&mut self, invert: bool) -> Result<(), Unsupported> {
        if self.pin.is_i2c_pin() {
            return Err(Unsupported);
        }

        self.pin.iocon().modify(|r| r.set_inv(invert));
        Ok(())
    }

    #[inline]
    pub fn set_glitch_filtering(&mut self, filter: bool) -> Result<(), Unsupported> {
        if !self.pin.is_adc_pin() {
            return Err(Unsupported);
        }

        self.pin.iocon().modify(|r| r.set_filter(filter));
        Ok(())
    }

    #[inline]
    pub fn set_open_drain(&mut self, od: bool) -> Result<(), Unsupported> {
        if self.pin.is_i2c_pin() {
            if od {
                return Ok(());
            } else {
                return Err(Unsupported);
            }
        }

        self.pin.iocon().modify(|r| r.set_od(od));
        Ok(())
    }

    #[inline]
    fn set_dir(&mut self, dir: bool) {
        self.pin
            .gpio_port()
            .dir()
            .modify(|r| r.set_p(self.pin() as _, dir))
    }

    #[inline]
    pub fn set_as_output(&mut self) {
        self.set_dir(true);
    }

    #[inline]
    pub fn set_as_input(&mut self) {
        self.set_dir(false);
    }

    #[inline]
    pub fn is_set_as_output(&self) -> bool {
        self.pin.gpio_port().dir().read().p(self.pin() as _)
    }

    #[inline]
    pub fn is_high(&self) -> bool {
        self.pin.gpio_port().pin().read().p(self.pin() as _)
    }

    #[inline]
    pub fn is_low(&self) -> bool {
        !self.is_high()
    }

    #[inline]
    pub fn get_input_level(&self) -> Level {
        match self.is_high() {
            true => Level::High,
            false => Level::Low,
        }
    }

    #[inline]
    pub fn set_low(&mut self) {
        self.pin.gpio_port().clr().write_value(Clr(self.bit()))
    }

    #[inline]
    pub fn set_high(&mut self) {
        self.pin.gpio_port().set().write_value(Set(self.bit()))
    }

    #[inline]
    pub fn set_output_level(&mut self, level: Level) {
        match level {
            Level::Low => self.set_low(),
            Level::High => self.set_high(),
        }
    }

    #[inline]
    pub fn is_set_high(&self) -> bool {
        self.pin.gpio_port().set().read().p(self.pin() as _)
    }

    #[inline]
    pub fn is_set_low(&self) -> bool {
        !self.is_set_high()
    }

    #[inline]
    pub fn get_output_level(&self) -> Level {
        match self.is_set_high() {
            false => Level::Low,
            true => Level::High,
        }
    }

    #[inline]
    pub fn toggle_output(&mut self) {
        self.pin.gpio_port().not().write_value(Not(self.bit()))
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Input<'d> {
    pin: Flex<'d>,
}

impl<'d> Input<'d> {
    #[inline]
    pub fn new(pin: Peri<'d, impl Pin>, pull: Pull) -> Result<Self, Unsupported> {
        let mut pin = Flex::new(pin);

        pin.set_pull(pull)?;
        pin.set_as_input();
        Ok(Self { pin })
    }

    #[inline]
    pub fn set_hysteresis(&mut self, enable: bool) -> Result<(), Unsupported> {
        self.pin.set_hysteresis(enable)
    }

    #[inline]
    pub fn set_inversion(&mut self, invert: bool) -> Result<(), Unsupported> {
        self.pin.set_input_inversion(invert)
    }

    #[inline]
    pub fn set_glitch_filtering(&mut self, filter: bool) -> Result<(), Unsupported> {
        self.pin.set_glitch_filtering(filter)
    }

    #[inline]
    pub fn set_pull(&mut self, pull: Pull) -> Result<(), Unsupported> {
        self.pin.set_pull(pull)
    }

    #[inline]
    pub fn get_level(&self) -> Level {
        self.pin.get_input_level()
    }

    #[inline]
    pub fn is_high(&self) -> bool {
        self.pin.is_high()
    }

    #[inline]
    pub fn is_low(&self) -> bool {
        self.pin.is_low()
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Output<'d> {
    pin: Flex<'d>,
}

impl<'d> Output<'d> {
    #[inline]
    pub fn new(pin: Peri<'d, impl Pin>, initial_output: Level) -> Self {
        let mut pin = Flex::new(pin);

        match initial_output {
            Level::Low => pin.set_low(),
            Level::High => pin.set_high(),
        }

        pin.set_as_output();
        Self { pin }
    }

    #[inline]
    pub fn set_open_drain(&mut self, od: bool) -> Result<(), Unsupported> {
        self.pin.set_open_drain(od)
    }

    #[inline]
    pub fn set_high(&mut self) {
        self.pin.set_high();
    }

    #[inline]
    pub fn set_low(&mut self) {
        self.pin.set_low()
    }

    #[inline]
    pub fn set_level(&mut self, level: Level) {
        self.pin.set_output_level(level);
    }

    #[inline]
    pub fn toggle(&mut self) {
        self.pin.toggle_output();
    }

    #[inline]
    pub fn is_set_high(&self) -> bool {
        self.pin.is_set_high()
    }

    #[inline]
    pub fn is_set_low(&self) -> bool {
        self.pin.is_set_low()
    }

    #[inline]
    pub fn get_level(&self) -> Level {
        self.pin.get_output_level()
    }
}

macro_rules! impl_pin {
    ($name:ident, $port:expr, $pin:expr, $pio_func:literal, $is_i2c_pin:literal, $is_adc_pin:literal) => {
        impl Pin for peripherals::$name {}
        impl SealedPin for peripherals::$name {
            #[inline]
            fn port_pin(&self) -> u8 {
                ($port as u8) << 5 | $pin
            }

            #[inline]
            fn pio_func(&self) -> u8 {
                $pio_func
            }

            #[inline]
            fn is_i2c_pin(&self) -> bool {
                $is_i2c_pin
            }

            #[inline]
            fn is_adc_pin(&self) -> bool {
                $is_adc_pin
            }
        }

        impl From<peripherals::$name> for crate::gpio::AnyPin {
            fn from(val: peripherals::$name) -> Self {
                Self {
                    port_pin: val.port_pin(),
                }
            }
        }
    };
}



impl_pin!(PIO0_0, Port::Port0, 0, 1, false, false);
impl_pin!(PIO0_1, Port::Port0, 1, 0, false, false);
impl_pin!(PIO0_2, Port::Port0, 2, 0, false, false);
impl_pin!(PIO0_3, Port::Port0, 3, 0, false, false);
impl_pin!(PIO0_4, Port::Port0, 4, 0, true, false);
impl_pin!(PIO0_5, Port::Port0, 5, 0, true, false);
impl_pin!(PIO0_6, Port::Port0, 6, 0, false, false);
impl_pin!(PIO0_7, Port::Port0, 7, 0, false, false);
impl_pin!(PIO0_8, Port::Port0, 8, 0, false, false);
impl_pin!(PIO0_9, Port::Port0, 9, 0, false, false);
impl_pin!(PIO0_10, Port::Port0, 10, 1, false, false);
impl_pin!(PIO0_11, Port::Port0, 11, 1, false, true);
impl_pin!(PIO0_12, Port::Port0, 12, 1, false, true);
impl_pin!(PIO0_13, Port::Port0, 13, 1, false, true);
impl_pin!(PIO0_14, Port::Port0, 14, 1, false, true);
impl_pin!(PIO0_15, Port::Port0, 15, 1, false, true);
impl_pin!(PIO0_16, Port::Port0, 16, 0, false, true);
impl_pin!(PIO0_17, Port::Port0, 17, 0, false, false);
impl_pin!(PIO0_18, Port::Port0, 18, 0, false, false);
impl_pin!(PIO0_19, Port::Port0, 19, 0, false, false);
impl_pin!(PIO0_20, Port::Port0, 20, 0, false, false);
impl_pin!(PIO0_21, Port::Port0, 21, 0, false, false);
impl_pin!(PIO0_22, Port::Port0, 22, 0, false, true);
impl_pin!(PIO0_23, Port::Port0, 23, 0, false, true);

#[cfg(feature = "lqfp64")]
impl_pin!(PIO1_0, Port::Port1, 0, 0, false, false);
#[cfg(feature = "lqfp64")]
impl_pin!(PIO1_1, Port::Port1, 1, 0, false, false);
#[cfg(feature = "lqfp64")]
impl_pin!(PIO1_2, Port::Port1, 2, 0, false, false);
#[cfg(feature = "lqfp64")]
impl_pin!(PIO1_3, Port::Port1, 3, 0, false, false);
#[cfg(feature = "lqfp64")]
impl_pin!(PIO1_4, Port::Port1, 4, 0, false, false);
#[cfg(any(feature = "lqfp64", feature = "tfbga48"))]
impl_pin!(PIO1_5, Port::Port1, 5, 0, false, false);
#[cfg(feature = "lqfp64")]
impl_pin!(PIO1_6, Port::Port1, 6, 0, false, false);
#[cfg(feature = "lqfp64")]
impl_pin!(PIO1_7, Port::Port1, 7, 0, false, false);
#[cfg(feature = "lqfp64")]
impl_pin!(PIO1_8, Port::Port1, 8, 0, false, false);
#[cfg(feature = "lqfp64")]
impl_pin!(PIO1_9, Port::Port1, 9, 0, false, false);
#[cfg(feature = "lqfp64")]
impl_pin!(PIO1_10, Port::Port1, 10, 0, false, false);
#[cfg(feature = "lqfp64")]
impl_pin!(PIO1_11, Port::Port1, 11, 0, false, false);
#[cfg(feature = "lqfp64")]
impl_pin!(PIO1_12, Port::Port1, 12, 0, false, false);
#[cfg(any(feature = "lqfp64", feature = "lqfp48", feature = "tfbga48"))]
impl_pin!(PIO1_13, Port::Port1, 13, 0, false, false);
#[cfg(any(feature = "lqfp64", feature = "lqfp48", feature = "tfbga48"))]
impl_pin!(PIO1_14, Port::Port1, 14, 0, false, false);
impl_pin!(PIO1_15, Port::Port1, 15, 0, false, false);
#[cfg(any(feature = "lqfp64", feature = "lqfp48", feature = "tfbga48"))]
impl_pin!(PIO1_16, Port::Port1, 16, 0, false, false);
#[cfg(feature = "lqfp64")]
impl_pin!(PIO1_17, Port::Port1, 17, 0, false, false);
#[cfg(feature = "lqfp64")]
impl_pin!(PIO1_18, Port::Port1, 18, 0, false, false);
impl_pin!(PIO1_19, Port::Port1, 19, 0, false, false);
#[cfg(any(feature = "lqfp64", feature = "lqfp48", feature = "tfbga48"))]
impl_pin!(PIO1_20, Port::Port1, 20, 0, false, false);
#[cfg(any(feature = "lqfp64", feature = "lqfp48", feature = "tfbga48"))]
impl_pin!(PIO1_21, Port::Port1, 21, 0, false, false);
#[cfg(any(feature = "lqfp64", feature = "lqfp48", feature = "tfbga48"))]
impl_pin!(PIO1_22, Port::Port1, 22, 0, false, false);
#[cfg(any(feature = "lqfp64", feature = "lqfp48", feature = "tfbga48"))]
impl_pin!(PIO1_23, Port::Port1, 23, 0, false, false);
#[cfg(any(feature = "lqfp64", feature = "lqfp48", feature = "tfbga48"))]
impl_pin!(PIO1_24, Port::Port1, 24, 0, false, false);
#[cfg(any(feature = "lqfp64", feature = "lqfp48", feature = "tfbga48"))]
impl_pin!(PIO1_25, Port::Port1, 25, 0, false, false);
#[cfg(any(feature = "lqfp64", feature = "lqfp48", feature = "tfbga48"))]
impl_pin!(PIO1_26, Port::Port1, 26, 0, false, false);
#[cfg(any(feature = "lqfp64", feature = "lqfp48", feature = "tfbga48"))]
impl_pin!(PIO1_27, Port::Port1, 27, 0, false, false);
#[cfg(any(feature = "lqfp64", feature = "lqfp48", feature = "tfbga48"))]
impl_pin!(PIO1_28, Port::Port1, 28, 0, false, false);
#[cfg(any(feature = "lqfp64", feature = "lqfp48", feature = "tfbga48"))]
impl_pin!(PIO1_29, Port::Port1, 29, 0, false, false);
#[cfg(feature = "lqfp48")]
impl_pin!(PIO1_31, Port::Port1, 31, 0, false, false);

#[cfg(feature = "eh02")]
mod eh02 {

    use super::*;
    use ::eh02 as embedded_hal;

    impl<'d> embedded_hal::digital::v2::InputPin for Input<'d> {
        type Error = Infallible;
    
        fn is_high(&self) -> Result<bool, Self::Error> {
            Ok(self.is_high())
        }
    
        fn is_low(&self) -> Result<bool, Self::Error> {
            Ok(self.is_low())
        }
    }

    impl<'d> embedded_hal::digital::v2::OutputPin for Output<'d> {
        type Error = Infallible;
    
        fn set_low(&mut self) -> Result<(), Self::Error> {
            self.set_low();
            Ok(())
        }
    
        fn set_high(&mut self) -> Result<(), Self::Error> {
            self.set_high();
            Ok(())
        }
    }

    impl<'d> embedded_hal::digital::v2::StatefulOutputPin for Output<'d> {
        fn is_set_high(&self) -> Result<bool, Self::Error> {
            Ok(self.is_set_high())
        }
    
        fn is_set_low(&self) -> Result<bool, Self::Error> {
            Ok(self.is_set_low())
        }
    }

    impl<'d> embedded_hal::digital::v2::InputPin for Flex<'d> {
        type Error = Infallible;
    
        fn is_high(&self) -> Result<bool, Self::Error> {
            Ok(self.is_high())
        }
    
        fn is_low(&self) -> Result<bool, Self::Error> {
            Ok(self.is_low())
        }
    }

    impl<'d> embedded_hal::digital::v2::OutputPin for Flex<'d> {
        type Error = Infallible;
    
        fn set_low(&mut self) -> Result<(), Self::Error> {
            self.set_low();
            Ok(())
        }
    
        fn set_high(&mut self) -> Result<(), Self::Error> {
            self.set_high();
            Ok(())
        }
    }

    impl<'d> embedded_hal::digital::v2::StatefulOutputPin for Flex<'d> {
        fn is_set_high(&self) -> Result<bool, Self::Error> {
            Ok(self.is_set_high())
        }
    
        fn is_set_low(&self) -> Result<bool, Self::Error> {
            Ok(self.is_set_low())
        }
    }

    impl<'d> embedded_hal::digital::v2::ToggleableOutputPin for Flex<'d> {
        type Error = Infallible;
    
        fn toggle(&mut self) -> Result<(), Self::Error> {
            self.toggle_output();
            Ok(())
        }
    }
}

#[cfg(feature = "eh10")]
mod eh10 {
    use super::*;
    use ::eh10 as embedded_hal;

    impl<'d> embedded_hal::digital::ErrorType for Input<'d> {
        type Error = Infallible;
    }

    impl<'d> embedded_hal::digital::InputPin for Input<'d> {
        fn is_high(&mut self) -> Result<bool, Self::Error> {
            Ok(Self::is_high(self))
        }
    
        fn is_low(&mut self) -> Result<bool, Self::Error> {
            Ok(Self::is_low(self))
        }
    }

    impl<'d> embedded_hal::digital::ErrorType for Output<'d> {
        type Error = Infallible;
    }

    impl<'d> embedded_hal::digital::OutputPin for Output<'d> {
        fn set_low(&mut self) -> Result<(), Self::Error> {
            Self::set_low(self);
            Ok(())
        }
    
        fn set_high(&mut self) -> Result<(), Self::Error> {
            Self::set_high(self);
            Ok(())
        }
    }

    impl<'d> embedded_hal::digital::StatefulOutputPin for Output<'d> {
        fn is_set_high(&mut self) -> Result<bool, Self::Error> {
            Ok(Self::is_set_high(self))
        }
    
        fn is_set_low(&mut self) -> Result<bool, Self::Error> {
            Ok(Self::is_set_low(self))
        }
        
        fn toggle(&mut self) -> Result<(), Self::Error> {
            Output::toggle(self);
            Ok(())
        }
    }

    impl<'d> embedded_hal::digital::ErrorType for Flex<'d> {
        type Error = Infallible;
    }

    impl<'d> embedded_hal::digital::InputPin for Flex<'d> {
        fn is_high(&mut self) -> Result<bool, Self::Error> {
            Ok(Self::is_high(self))
        }
    
        fn is_low(&mut self) -> Result<bool, Self::Error> {
            Ok(Self::is_low(self))
        }
    }

    impl<'d> embedded_hal::digital::OutputPin for Flex<'d> {
        fn set_low(&mut self) -> Result<(), Self::Error> {
            Self::set_low(self);
            Ok(())
        }
    
        fn set_high(&mut self) -> Result<(), Self::Error> {
            Self::set_high(self);
            Ok(())
        }
    }
    
    impl<'d> embedded_hal::digital::StatefulOutputPin for Flex<'d> {
        fn is_set_high(&mut self) -> Result<bool, Self::Error> {
            Ok(Self::is_set_high(self))
        }
    
        fn is_set_low(&mut self) -> Result<bool, Self::Error> {
            Ok(Self::is_set_low(self))
        }
        
        fn toggle(&mut self) -> Result<(), Self::Error> {
            Self::toggle_output(self);
            Ok(())
        }
    }
}
