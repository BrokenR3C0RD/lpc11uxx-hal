#[repr(C)]
pub struct _DivReturn<T> {
    pub quot: T,
    pub rem: T,
}

type _Idiv<T> = extern "C" fn(T, T) -> T;
type _Idivmod<T> = extern "C" fn(T, T) -> _DivReturn<T>;
type _CmdResp = unsafe extern "C" fn(cmd: *const u32, resp: *mut u32);

type _UsbdRom = ();

#[repr(C)]
pub struct _RomDiv {
    pub sidiv: _Idiv<i32>,
    pub uidiv: _Idiv<u32>,
    pub sidivmod: _Idivmod<i32>,
    pub uidivmod: _Idivmod<u32>,
}

unsafe impl Sync for _RomDiv {}

#[repr(C)]
pub struct _Pwrd {
    set_pll: _CmdResp,
    set_power: _CmdResp,
}

#[repr(C)]
struct _Rom {
    usbd: *const _UsbdRom,
    _p_dev2: *const (),
    _p_dev3: *const (),
    pwrd: *const _Pwrd,
    romdiv: *const _RomDiv,
    _p_dev6: *const (),
    _p_dev7: *const (),
    _p_dev8: *const (),
}

const ROM: *const *const _Rom = 0x1FFF_1FF8 as _;
const IAP: *const () = 0x1FFF_1FF1 as _;

pub struct RomDrivers;

impl RomDrivers {
    #[inline(always)]
    fn rom_table() -> &'static _Rom {
        unsafe { &**ROM }
    }

    #[inline(always)]
    pub fn usb() -> &'static _UsbdRom {
        unsafe { &*(Self::rom_table().usbd) }
    }

    #[inline(always)]
    pub fn power() -> &'static _Pwrd {
        unsafe { &*(Self::rom_table().pwrd) }
    }

    #[inline(always)]
    pub fn intdiv() -> &'static _RomDiv {
        unsafe { &*(Self::rom_table().romdiv) }
    }
}

#[inline(always)]
/// # Safety
/// This can overwrite running code and cause undefined behavior.
/// Always run code that can modify flash from RAM.
pub unsafe fn iap_entry(command_param: &[u32], status_result: &mut [u32]) {
    unsafe {
        core::mem::transmute::<*const (), _CmdResp>(IAP)(
            command_param.as_ptr(),
            status_result.as_mut_ptr(),
        )
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(C, u32, align(4))]
pub enum IapCommand {
    PrepareSectorsForWriteOperation {
        first: u32,
        last: u32,
    } = 50,
    CopyRamToFlash {
        dst: *mut (),
        src: *const (),
        nbytes: u32,
        cclk_khz: u32,
    } = 51,
    EraseSectors {
        first: u32,
        last: u32,
        cclk_khz: u32,
    } = 52,
    BlankCheckSectors {
        first: u32,
        last: u32,
    } = 53,
    ReadPartId {} = 54,
    ReadBootCodeVersion = 55,
    Compare {
        dst: *const u32,
        src: *const u32,
        nbytes: u32,
    } = 56,
    ReinvokeIsp {} = 57,
    ReadUid {} = 58,
    #[cfg(any(
        feature = "lpc11u34",
        feature = "lpc11u35",
        feature = "lpc11u36",
        feature = "lpc11u37"
    ))]
    ErasePage {
        first_page: u32,
        last_page: u32,
        cclk_khz: u32,
    } = 59,

    WriteEeprom {
        eeprom_dst: u32,
        src: *const u8,
        nbytes: u32,
        cclk_khz: u32,
    } = 61,
    ReadEeprom {
        eeprom_src: u32,
        dst: *mut u8,
        nbytes: u32,
        cclk_khz: u32,
    } = 62,
}

impl From<IapCommand> for [u32; 5] {
    #[inline]
    fn from(val: IapCommand) -> Self {
        unsafe { core::mem::transmute(val) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(C, u32, align(4))]
pub enum IapResult<T: Sized> {
    Success(T) = 0,
    InvalidCommand = 1,
    SrcAddrError = 2,
    DstAddrError = 3,
    SrcAddrNotMapped = 4,
    DstAddrNotMapped = 5,
    CountError = 6,
    InvalidSector = 7,
    SectorNotBlank { first_offset: usize, contents: u32 } = 8,
    SectorNotPreparedForWriteOperation = 9,
    CompareError { first_offset: usize } = 10,
    Busy = 11,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(C, align(4))]
pub struct PartId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(C, align(4))]
pub struct BootCodeVersion(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(C, align(4))]
pub struct Uid(pub [u32; 4]);

macro_rules! impl_iap_functions {
    () => {};

    (
        $(#[$meta:meta])*
        $variant:ident: fn $name:ident ($($arg:ident: $argty:ty),*$(,)?) -> !,
        $($tt:tt)*
    ) => {
        $(#[$meta])*
        #[inline(always)]
        pub fn $name($($arg: $argty),*) -> ! {
            let cmd: [u32; 5] = IapCommand::$variant {
                $($arg: $arg as _),*
            }.into();

            unsafe {
                iap_entry(&cmd, &mut []);
                core::hint::unreachable_unchecked();
            }
        }
        impl_iap_functions!($($tt)*);
    };

    (
        $(#[$meta:meta])*
        $variant:ident: fn $name:ident ($($arg:ident: $argty:ty),*$(,)?) -> IapResult<$ty:ty>,
        $($tt:tt)*
    ) => {
        $(#[$meta])*
        #[inline(always)]
        pub fn $name($($arg: $argty),*) -> IapResult<$ty> {
            let cmd: [u32; 5] = IapCommand::$variant {
                $($arg: $arg as _),*
            }.into();
            let mut resp = [0u32; size_of::<IapResult<$ty>>() / 4];

            unsafe {
                iap_entry(&cmd, &mut resp);
                let ret = core::mem::transmute::<[u32; size_of::<IapResult<$ty>>() / 4], IapResult<$ty>>(resp);
                ret
            }
        }

        impl_iap_functions!($($tt)*);
    };

    (
        $(#[$meta:meta])*
        $variant:ident: unsafe fn $name:ident ($($arg:ident: $argty:ty),*$(,)?) -> IapResult<$ty:ty>,
        $($tt:tt)*
    ) => {
        $(#[$meta])*
        #[inline(always)]
        pub unsafe fn $name($($arg: $argty),*) -> IapResult<$ty> {
            let cmd: [u32; 5] = IapCommand::$variant {
                $($arg: $arg as _),*
            }.into();
            let mut resp = [0u32; size_of::<IapResult<$ty>>() / 4];

            unsafe {
                iap_entry(&cmd, &mut resp);
                let ret = core::mem::transmute::<[u32; size_of::<IapResult<$ty>>() / 4], IapResult<$ty>>(resp);
                ret
            }
        }
        impl_iap_functions!($($tt)*);

    };

    (
        $(#[$meta:meta])*
        $variant:ident: fn $name:ident ($($arg:ident: $argty:ty),*$(,)?) -> $ty:ty,
        $($tt:tt)*
    ) => {
        $(#[$meta])*
        #[inline(always)]
        pub fn $name($($arg: $argty),*) -> $ty {
            let cmd: [u32; 5] = IapCommand::$variant {
                $($arg: $arg as _),*
            }.into();
            let mut resp = [0u32; size_of::<IapResult<$ty>>() / 4];

            unsafe {
                iap_entry(&cmd, &mut resp);
                let IapResult::Success(ret) = core::mem::transmute::<[u32; size_of::<IapResult<$ty>>() / 4], IapResult<$ty>>(resp)
                    else { core::hint::unreachable_unchecked() };
                ret
            }
        }
        impl_iap_functions!($($tt)*);

    };

    (
        $(#[$meta:meta])*
        $variant:ident: unsafe fn $name:ident ($($arg:ident: $argty:ty),*$(,)?) -> $ty:ty,
        $($tt:tt)*
    ) => {
        $(#[$meta])*
        #[inline(always)]
        pub unsafe fn $name($($arg: $argty),*) -> $ty {
            let cmd: [u32; 5] = IapCommand::$variant {
                $($arg: $arg as _),*
            }.into();
            let mut resp = [0u32; size_of::<IapResult<$ty>>() / 4];

            unsafe {
                iap_entry(&cmd, &mut resp);
                let IapResult::Success(ret) = core::mem::transmute::<[u32; size_of::<IapResult<$ty>>() / 4], IapResult<$ty>>(resp)
                    else { core::hint::unreachable_unchecked() };
                ret
            }
        }
        impl_iap_functions!($($tt)*);
    };
}

impl_iap_functions! {
    /// Prepare sector(s) for write operation
    /// 
    /// This command must be executed before executing "Copy RAM to flash" or "Erase
    /// Sector(s)" command. Successful execution of the "Copy RAM to flash" or "Erase
    /// Sector(s)" command causes relevant sectors to be protected again. The boot
    /// sector can not be prepared by this command. To prepare a single sector use the
    /// same "Start" and "End" sector numbers.
    PrepareSectorsForWriteOperation: fn prepare_sectors_for_write(first: u32, last: u32) -> IapResult<()>,

    /// Copy RAM to flash
    /// 
    /// This command is used to program the flash memory. The affected sectors should
    /// be prepared first by calling "Prepare Sector for Write Operation" command. The
    /// affected sectors are automatically protected again once the copy command is
    /// successfully executed. The boot sector can not be written by this command. Also
    /// see Section 20.6 for the number of bytes that can be written.
    CopyRamToFlash: unsafe fn copy_ram_to_flash(dst: *mut (), src: *const (), nbytes: u32, cclk_khz: u32) -> IapResult<()>,

    /// Erase Sector(s)
    /// 
    /// This command is used to erase a sector or multiple sectors of on-chip flash
    /// memory. The boot sector can not be erased by this command. To erase a single
    /// sector use the same "Start" and "End" sector numbers.
    EraseSectors: unsafe fn erase_sectors(first: u32, last: u32, cclk_khz: u32) -> IapResult<()>,

    /// Blank check sector(s)
    /// 
    /// This command is used to blank check a sector or multiple sectors of on-chip flash
    /// memory. To blank check a single sector use the same "Start" and "End" sector
    /// numbers.
    BlankCheckSectors: fn blank_check_sectors(first: u32, last: u32) -> IapResult<()>,

    /// Read Part Identification number
    /// 
    /// This command is used to read the part identification number.
    ReadPartId: fn read_part_id() -> PartId,

    /// Read Boot code version number
    /// 
    /// This command is used to read the boot code version number.
    ReadBootCodeVersion: fn read_boot_code_version() -> BootCodeVersion,

    /// Compare <address1> <address2> <no of bytes>
    /// This command is used to compare the memory contents at two locations.
    /// 
    /// **The result may not be correct when the source or destination includes any
    /// of the first 512 bytes starting from address zero. The first 512 bytes can be
    /// re-mapped to RAM.**
    Compare: fn compare(dst: *const u32, src: *const u32, nbytes: u32) -> IapResult<()>,

    /// Reinvoke ISP
    /// 
    /// This command is used to invoke the bootloader in ISP mode. It maps boot
    /// vectors, sets PCLK = CCLK, configures UART pins RXD and TXD, resets
    /// counter/timer CT32B1 and resets the U0FDR (see Table 233). This command
    /// may be used when a valid user program is present in the internal flash memory
    /// and the PIO0_1 pin is not accessible to force the ISP mode.
    ReinvokeIsp: fn reinvoke_isp() -> !,

    /// Read UID
    /// 
    /// This command is used to read the unique ID.
    ReadUid: fn read_uid() -> Uid,

    #[cfg(any(
        feature = "lpc11u34",
        feature = "lpc11u35",
        feature = "lpc11u36",
        feature = "lpc11u37"
    ))]
    /// Erase page
    /// 
    /// This command is used to erase a page or multiple pages of on-chip flash memory.
    /// To erase a single page use the same "start" and "end" page numbers. See
    /// Table 343 for list of parts that implement this command.
    ErasePage: unsafe fn erase_page(first_page: u32, last_page: u32, cclk_khz: u32) -> IapResult<()>,

    /// Write EEPROM
    /// 
    /// Data is copied from the RAM address to the EEPROM address.
    /// 
    /// **Remark**: The top 64 bytes of the 4 kB EEPROM memory are reserved and
    /// cannot be written to. The entire EEPROM is writable for smaller EEPROM sizes.
    WriteEeprom: unsafe fn write_eeprom(eeprom_dst: u32, src: *const u8, nbytes: u32, cclk_khz: u32) -> IapResult<()>,

    /// Read EEPROM
    /// 
    /// Data is copied from the EEPROM address to the RAM address.
    ReadEeprom: unsafe fn read_eeprom(eeprom_src: u32, dst: *mut u8, nbytes: u32, cclk_khz: u32) -> IapResult<()>,
}
