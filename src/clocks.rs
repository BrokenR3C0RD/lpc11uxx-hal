use crate::pac;

use core::{
    num::{NonZeroU8, NonZeroU32},
    sync::atomic::AtomicU32,
};
use pac::syscon::vals::{MainclkselSel, PllclkselSel, UsbclkselSel};

struct Clocks {
    sysosc: AtomicU32,
    wdosc: AtomicU32,
    sys_pll: AtomicU32,
    usb_pll: AtomicU32,
    mainclk: AtomicU32,
    usb_pclk: AtomicU32,
    ssp0_pclk: AtomicU32,
    ssp1_pclk: AtomicU32,
    usart_pclk: AtomicU32,
}

static CLOCKS: Clocks = Clocks {
    sysosc: AtomicU32::new(0),
    wdosc: AtomicU32::new(0),
    sys_pll: AtomicU32::new(0),
    usb_pll: AtomicU32::new(0),
    mainclk: AtomicU32::new(0),
    usb_pclk: AtomicU32::new(0),
    ssp0_pclk: AtomicU32::new(0),
    ssp1_pclk: AtomicU32::new(0),
    usart_pclk: AtomicU32::new(0),
};

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PllClkSrc {
    Irc = PllclkselSel::IRC as _,
    Sysosc = PllclkselSel::SYSOSC as _,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum MainClkSrc {
    Irc = MainclkselSel::IRC as _,
    SysOsc = MainclkselSel::PLL_IN as _,
    WdOsc = MainclkselSel::WDTOSC as _,
    SysPll = MainclkselSel::PLL_OUT as _,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum UsbClkSrc {
    MainClk = UsbclkselSel::MAINCLK as _,
    UsbPll = UsbclkselSel::USB_PLL_OUT as _,
}

pub struct ClockConfig {
    pub irc: IrcConfig,
    pub sysosc_khz: Option<NonZeroU32>,
    pub wdosc: Option<WdOscConfig>,
    pub mainclk: MainClkConfig,
    pub sys_pll: Option<PllConfig>,
    pub usb_pll: Option<PllConfig>,
    pub usb_pclk: Option<UsbClkConfig>,
    pub ssp0_pclk_divider: Option<NonZeroU8>,
    pub ssp1_pclk_divider: Option<NonZeroU8>,
    pub usart_pclk_divider: Option<NonZeroU8>,
}

pub enum ClockError {
    /// Requested system clock out of range
    SysClkOutOfRange,
    /// Requested watchdog oscillator frequency out of range
    WdOscOutOfRange,
    /// Could not find valid PLL parameters for system PLL.
    InvalidSysPllParameters,
    /// Could not find valid PLL parameters for USB PLL.
    InvalidUsbPllParameters,
    /// USB clock is not 48MHz
    UsbClkOutOfRange,
    /// System PLL failed to lock within the timeout period.
    SysPllLockTimedOut,
    /// USB PLL failed to lock within the timeout period.
    UsbPllLockTimedOut,
}

impl Default for ClockConfig {
    fn default() -> Self {
        Self::irc_12mhz()
    }
}

impl ClockConfig {
    const fn default() -> Self {
        Self::irc_12mhz()
    }

    /// Directly use the IRC as the main clock's source.
    pub const fn irc_12mhz() -> Self {
        Self {
            irc: IrcConfig::Enabled,
            sysosc_khz: None,
            wdosc: None,
            mainclk: MainClkConfig {
                divider: NonZeroU8::new(1).unwrap(),
                source: MainClkSrc::Irc,
            },
            sys_pll: None,
            usb_pll: None,
            usb_pclk: None,
            ssp0_pclk_divider: None,
            ssp1_pclk_divider: None,
            usart_pclk_divider: None,
        }
    }

    /// Use the IRC to drive the system PLL to 24MHz, and use the system PLL as the main clock's source.
    pub const fn irc_24mhz() -> Self {
        Self {
            mainclk: MainClkConfig {
                source: MainClkSrc::SysPll,
                divider: NonZeroU8::new(1).unwrap(),
            },
            sys_pll: Some(PllConfig {
                source: PllClkSrc::Irc,
                m: 2,
                p: 4,
            }),
            ..Self::default()
        }
    }

    /// Use the IRC to drive the system PLL to 24MHz, and use the system PLL as the main clock's source.
    pub const fn irc_48mhz() -> Self {
        Self {
            mainclk: MainClkConfig {
                source: MainClkSrc::SysPll,
                divider: NonZeroU8::new(1).unwrap(),
            },
            sys_pll: Some(PllConfig {
                source: PllClkSrc::Irc,
                m: 4,
                p: 2,
            }),
            ..Self::default()
        }
    }

    /// Use an external cystal oscillator as the main clock source.
    pub const fn crystal_oscillator(khz: u32) -> Self {
        Self {
            irc: IrcConfig::Disabled,
            sysosc_khz: Some(NonZeroU32::new(khz).expect("hz must be non-zero")),
            mainclk: MainClkConfig {
                source: MainClkSrc::SysOsc,
                divider: NonZeroU8::new(1).unwrap(),
            },
            ..Self::default()
        }
    }

    #[inline]
    pub const fn irc_khz(&self) -> Option<u32> {
        match self.irc {
            IrcConfig::Disabled => None,
            IrcConfig::Enabled => Some(12_000),
        }
    }

    #[inline]
    pub const fn sysosc_khz(&self) -> Option<u32> {
        match self.sysosc_khz {
            None => None,
            Some(v) => Some(v.get()),
        }
    }

    #[inline]
    pub const fn wdosc_khz(&self) -> Option<u32> {
        ::core::todo!()
    }

    #[inline]
    pub const fn syspll_khz(&self) -> Option<u32> {
        match self.sys_pll {
            None => None,
            Some(PllConfig {
                source: PllClkSrc::Irc,
                m,
                p: _,
            }) => Some(self.irc_khz().expect("irc must be enabled") * m as u32),
            Some(PllConfig {
                source: PllClkSrc::Sysosc,
                m,
                p: _,
            }) => Some(self.sysosc_khz().expect("sysosc_khz must be set") * m as u32),
        }
    }

    #[inline]
    pub const fn mainclk_src_khz(&self) -> u32 {
        match self.mainclk.source {
            MainClkSrc::Irc => self.irc_khz().expect("irc must be enabled"),
            MainClkSrc::SysOsc => self.sysosc_khz().expect("sysosc_khz must be set"),
            MainClkSrc::SysPll => self.syspll_khz().expect("system pll must be configured"),
            MainClkSrc::WdOsc => ::core::todo!(),
        }
    }

    #[inline]
    const fn mainclk_is_sysosc_sourced(&self) -> bool {
        match self.mainclk.source {
            MainClkSrc::SysOsc => true,
            MainClkSrc::SysPll => matches!(
                self.sys_pll,
                Some(PllConfig {
                    source: PllClkSrc::Sysosc,
                    ..
                })
            ),
            _ => false,
        }
    }

    #[inline]
    pub const fn mainclk_khz(&self) -> u32 {
        self.mainclk_src_khz() / (self.mainclk.divider.get() as u32)
    }

    #[inline]
    pub const fn usbpll_khz(&self) -> Option<u32> {
        match self.usb_pll {
            None => None,
            Some(PllConfig {
                source: PllClkSrc::Irc,
                m,
                p: _,
            }) => Some(self.irc_khz().expect("irc must be enabled") * m as u32),
            Some(PllConfig {
                source: PllClkSrc::Sysosc,
                m,
                p: _,
            }) => Some(self.sysosc_khz().expect("sysosc_khz must be set") * m as u32),
        }
    }

    #[inline]
    pub const fn usbclk_khz(&self) -> Option<u32> {
        match self.usb_pclk {
            None => None,
            Some(UsbClkConfig {
                source: UsbClkSrc::MainClk,
                divider,
            }) => Some(self.mainclk_khz() / (divider.get() as u32)),
            Some(UsbClkConfig {
                source: UsbClkSrc::UsbPll,
                divider,
            }) => Some(
                self.usbpll_khz().expect("usb pll must be configured") / (divider.get() as u32),
            ),
        }
    }

    #[inline]
    pub const fn ssp0_pclk_khz(&self) -> Option<u32> {
        match self.ssp0_pclk_divider {
            None => None,
            Some(divider) => Some(self.mainclk_khz() / (divider.get() as u32)),
        }
    }

    #[inline]
    pub const fn ssp1_pclk_khz(&self) -> Option<u32> {
        match self.ssp1_pclk_divider {
            None => None,
            Some(divider) => Some(self.mainclk_khz() / (divider.get() as u32)),
        }
    }

    #[inline]
    pub const fn usart_pclk_khz(&self) -> Option<u32> {
        match self.usart_pclk_divider {
            None => None,
            Some(divider) => Some(self.mainclk_khz() / (divider.get() as u32)),
        }
    }

    #[inline]
    pub const fn enable_usb_fs(mut self) -> Self {
        // We need to target 48MHz
        const TARGET_KHZ: u32 = 48_000;

        // First, check mainclk_src, since then we wouldn't need the USB PLL
        let mainclk_src_khz = self.mainclk_src_khz();
        let (quot, rem) = (TARGET_KHZ / mainclk_src_khz, TARGET_KHZ % mainclk_src_khz);
        if self.mainclk_is_sysosc_sourced() && rem == 0 && quot < 256 {
            self.usb_pll = None;
            self.usb_pclk = Some(UsbClkConfig {
                divider: NonZeroU8::new(quot as u8).unwrap(),
                source: UsbClkSrc::MainClk,
            });
            return self;
        }

        // Next, check if we have a system oscillator
        if let Some(sysosc_khz) = self.sysosc_khz()
            && let Some((settings, divider)) =
                PllConfig::calculate_with_divider(PllClkSrc::Sysosc, sysosc_khz, TARGET_KHZ)
        {
            self.usb_pll = Some(settings);
            self.usb_pclk = Some(UsbClkConfig {
                divider: NonZeroU8::new(divider).unwrap(),
                source: UsbClkSrc::UsbPll,
            });
            return self;
        }

        ::core::panic!("Could not determine clock parameters for full speed USB operation. Either a system oscillator is not configured, or it can not be converted to 48MHz.")
    }

    pub const fn enable_ssp0(mut self, target_khz: u32) -> Self {
        let mainclk_khz = self.mainclk_khz();
        if mainclk_khz < target_khz {
            ::core::panic!("SSP0 out of range");
        }

        let divider = self.mainclk_khz() / target_khz;
        self.ssp0_pclk_divider = Some(NonZeroU8::new(divider as u8).unwrap());
        self
    }
}

#[derive(PartialEq, Eq)]
pub enum IrcConfig {
    Disabled,
    Enabled,
}

pub struct SysoscConfig {
    pub frequency: u32,
}

pub struct WdOscConfig {
    pub divider: NonZeroU8,
    pub analog_clock: u32,
}

pub struct PllConfig {
    pub source: PllClkSrc,
    pub m: u8,
    pub p: u8,
}

impl PllConfig {
    #[inline]
    pub const fn calculate(source: PllClkSrc, input_khz: u32, target_khz: u32) -> Option<Self> {
        if input_khz > target_khz {
            return None;
        }

        let (m, rem) = ((target_khz / input_khz) as u8, target_khz % input_khz);

        if rem != 0 || m == 0 || m > 32 {
            return None;
        }

        let mut p = 1u8;
        // User manual: 156MHz <= F_CCO <= 320MHz
        while p <= 8 {
            let cco = 2 * (p as u32) * target_khz;
            if 156_000 <= cco && cco <= 320_000 {
                return Some(Self { source, m, p });
            }
            p <<= 1;
        }

        None
    }

    #[inline]
    pub const fn calculate_with_divider(
        source: PllClkSrc,
        input_khz: u32,
        target_khz: u32,
    ) -> Option<(Self, u8)> {
        let mut div = 1u8;

        loop {
            let target_khz = target_khz * div as u32;
            if input_khz <= target_khz
                && let Some(settings) = Self::calculate(source, input_khz, target_khz)
            {
                return Some((settings, div));
            }

            let Some(new_div) = div.checked_add(1) else {
                return None;
            };
            div = new_div
        }
    }
}

pub struct MainClkConfig {
    pub source: MainClkSrc,
    pub divider: NonZeroU8,
}

impl Default for MainClkConfig {
    fn default() -> Self {
        Self {
            source: MainClkSrc::Irc,
            divider: NonZeroU8::new(1).unwrap(),
        }
    }
}

pub struct UsbClkConfig {
    pub divider: NonZeroU8,
    pub source: UsbClkSrc,
}
