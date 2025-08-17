#![macro_use]

// Credit: modified from `rp-hal` (also licensed Apache+MIT)
// https://github.com/rp-rs/rp-hal/blob/main/rp2040-hal/src/intrinsics.rs

/// Generate a series of aliases for an intrinsic function.
macro_rules! intrinsics_aliases {
    (
        extern $abi:tt fn $name:ident( $($argname:ident: $ty:ty),* ) -> $ret:ty,
    ) => {};
    (
        extern $abi:tt fn $name:ident( $($argname:ident: $ty:ty),* ) -> $ret:ty,
    ) => {};

    (
        extern $abi:tt fn $name:ident( $($argname:ident: $ty:ty),* ) -> $ret:ty,
        $alias:ident
        $($rest:ident)*
    ) => {
        #[cfg(all(target_arch = "arm", feature = "intrinsics"))]
        intrinsics! {
            extern $abi fn $alias( $($argname: $ty),* ) -> $ret {
                unsafe { $name($($argname),*) }
            }
        }

        intrinsics_aliases! {
            extern $abi fn $name( $($argname: $ty),* ) -> $ret,
            $($rest)*
        }
    };

    (
        extern $abi:tt fn $name:ident( $($argname:ident: $ty:ty),* ) -> $ret:ty,
        $alias:ident
        $($rest:ident)*
    ) => {
        #[cfg(all(target_arch = "arm", feature = "intrinsics"))]
        intrinsics! {
            unsafe extern $abi fn $alias( $($argname: $ty),* ) -> $ret {
                unsafe { $name($($argname),*) }
            }
        }

        intrinsics_aliases! {
            unsafe extern $abi fn $name( $($argname: $ty),* ) -> $ret,
            $($rest)*
        }
    };
}

/// The macro used to define overridden intrinsics.
///
/// This is heavily inspired by the macro used by compiler-builtins.  The idea
/// is to abstract anything special that needs to be done to override an
/// intrinsic function.  Intrinsic generation is disabled for non-ARM targets
/// so things like CI and docs generation do not have problems.  Additionally
/// they can be disabled by disabling the crate feature `intrinsics` for
/// testing or comparing performance.
///
/// Like the compiler-builtins macro, it accepts a series of functions that
/// looks like normal Rust code:
///
/// ```rust,ignore
/// intrinsics! {
///     extern "C" fn foo(a: i32) -> u32 {
///         // ...
///     }
///     #[nonstandard_attribute]
///     extern "C" fn bar(a: i32) -> u32 {
///         // ...
///     }
/// }
/// ```
///
/// Each function can also be decorated with nonstandard attributes to control
/// additional behaviour:
///
/// * `slower_than_default` - indicates that the override is slower than the
///   default implementation.  Currently this just disables the override
///   entirely.
/// * `alias` - accepts a list of names to alias the intrinsic to.
/// * `aeabi` - accepts a list of ARM EABI names to alias to.
///
macro_rules! intrinsics {
    () => {};

    (
        #[slower_than_default]
        $(#[$($attr:tt)*])*
        extern $abi:tt fn $name:ident( $($argname:ident: $ty:ty),* ) $(-> $ret:ty)? {
            $($body:tt)*
        }

        $($rest:tt)*
    ) => {
        // Not exported, but defined so the actual implementation is
        // considered used
        #[allow(dead_code)]
        fn $name( $($argname: $ty),* ) $(-> $ret)? {
            $($body)*
        }

        intrinsics!($($rest)*);
    };

    (
        #[alias = $($alias:ident),*]
        $(#[$($attr:tt)*])*
        extern $abi:tt fn $name:ident( $($argname:ident: $ty:ty),* ) $(-> $ret:ty)? {
            $($body:tt)*
        }

        $($rest:tt)*
    ) => {
        intrinsics! {
            $(#[$($attr)*])*
            extern $abi fn $name( $($argname: $ty),* ) $(-> $ret)? {
                $($body)*
            }
        }

        intrinsics_aliases! {
            extern $abi fn $name( $($argname: $ty),* ) $(-> $ret)?,
            $($alias) *
        }

        intrinsics!($($rest)*);
    };

    (
        #[alias = $($alias:ident),*]
        $(#[$($attr:tt)*])*
        extern $abi:tt fn $name:ident( $($argname:ident: $ty:ty),* ) $(-> $ret:ty)? {
            $($body:tt)*
        }

        $($rest:tt)*
    ) => {
        intrinsics! {
            $(#[$($attr)*])*
            unsafe extern $abi fn $name( $($argname: $ty),* ) $(-> $ret)? {
                $($body)*
            }
        }

        intrinsics_aliases! {
            unsafe extern $abi fn $name( $($argname: $ty),* ) $(-> $ret)?,
            $($alias) *
        }

        intrinsics!($($rest)*);
    };

    (
        #[aeabi = $($alias:ident),*]
        $(#[$($attr:tt)*])*
        extern $abi:tt fn $name:ident( $($argname:ident: $ty:ty),* ) $(-> $ret:ty)? {
            $($body:tt)*
        }

        $($rest:tt)*
    ) => {
        intrinsics! {
            $(#[$($attr)*])*
            extern $abi fn $name( $($argname: $ty),* ) $(-> $ret)? {
                $($body)*
            }
        }

        intrinsics_aliases! {
            extern "aapcs" fn $name( $($argname: $ty),* ) $(-> $ret)?,
            $($alias) *
        }

        intrinsics!($($rest)*);
    };

    (
        $(#[$($attr:tt)*])*
        extern $abi:tt fn $name:ident( $($argname:ident: $ty:ty),* ) -> $($ret:ty)? {
            $($body:tt)*
        }

        $($rest:tt)*
    ) => {
        #[cfg(all(target_arch = "arm", feature = "intrinsics"))]
        $(#[$($attr)*])*
        unsafe extern $abi fn $name( $($argname: $ty),* ) $(-> $ret)? {
            $($body)*
        }

        #[cfg(all(target_arch = "arm", feature = "intrinsics"))]
        mod $name {
            #[unsafe(no_mangle)]
            $(#[$($attr)*])*
            pub unsafe extern $abi fn $name( $($argname: $ty),* ) $(-> $ret)? {
                unsafe { super::$name($($argname),*) }
            }
        }

        // Not exported, but defined so the actual implementation is
        // considered used
        #[cfg(not(all(target_arch = "arm", feature = "intrinsics")))]
        #[allow(dead_code)]
        fn $name( $($argname: $ty),* ) $(-> $ret)? {
            $($body)*
        }

        intrinsics!($($rest)*);
    };

    (
        $(#[$($attr:tt)*])*
        extern $abi:tt fn $name:ident( $($argname:ident: $ty:ty),* ) $(-> $ret:ty)? {
            $($body:tt)*
        }

        $($rest:tt)*
    ) => {
        #[cfg(all(target_arch = "arm", feature = "intrinsics"))]
        $(#[$($attr)*])*
        unsafe extern $abi fn $name( $($argname: $ty),* ) $(-> $ret)? {
            $($body)*
        }

        #[cfg(all(target_arch = "arm", feature = "intrinsics"))]
        mod $name {
            #[unsafe(no_mangle)]
            $(#[$($attr)*])*
            pub unsafe extern $abi fn $name( $($argname: $ty),* ) $(-> $ret)? {
                unsafe { super::$name($($argname),*) }
            }
        }

        // Not exported, but defined so the actual implementation is
        // considered used
        #[cfg(not(all(target_arch = "arm", feature = "intrinsics")))]
        #[allow(dead_code)]
        unsafe fn $name( $($argname: $ty),* ) $(-> $ret)? {
            $($body)*
        }

        intrinsics!($($rest)*);
    };
}

intrinsics! {
    #[aeabi = __aeabi_udiv]
    extern "C" fn __udivsi3(n: u32, d: u32) -> u32 {
        (crate::rom::RomDrivers::intdiv().uidiv)(n, d)
    }

    #[aeabi = __aeabi_idiv]
    extern "C" fn __divsi3(n: i32, d: i32) -> i32 {
        (crate::rom::RomDrivers::intdiv().sidiv)(n, d)
    }

    extern "C" fn __udivmodsi4(n: u32, d: u32, rem: Option<&mut u32>) -> u32 {
        let res = (crate::rom::RomDrivers::intdiv().uidivmod)(n, d);
        if let Some(rem) = rem {
            *rem = res.rem;
        }
        res.quot
    }

    extern "C" fn __divmodsi4(n: i32, d: i32, rem: Option<&mut i32>) -> i32 {
        let res = (crate::rom::RomDrivers::intdiv().sidivmod)(n, d);
        if let Some(rem) = rem {
            *rem = res.rem;
        }
        res.quot
    }
}

/// Credit: taken/modified from compiler-builtins
/// https://github.com/rust-lang/compiler-builtins/blob/master/compiler-builtins/src/arm.rs
#[cfg(all(target_arch = "arm", feature = "intrinsics"))]
mod aeabi {
    #[unsafe(no_mangle)]
    #[unsafe(naked)]
    pub unsafe extern "custom" fn __aeabi_uidivmod() {
        core::arch::naked_asm!(
            "push {{lr}}",
            "sub sp, sp, #4",
            "mov r2, sp",
            "bl {trampoline}",
            "ldr r1, [sp]",
            "add sp, sp, #4",
            "pop {{pc}}",
            trampoline = sym crate::intrinsics::__udivmodsi4
        );
    }

    #[unsafe(no_mangle)]
    #[unsafe(naked)]
    pub unsafe extern "custom" fn __aeabi_idivmod() {
        core::arch::naked_asm!(
            "push {{lr}}",
            "sub sp, sp, #4",
            "mov r2, sp",
            "bl {trampoline}",
            "ldr r1, [sp]",
            "add sp, sp, #4",
            "pop {{pc}}",
            trampoline = sym crate::intrinsics::__divmodsi4
        );
    }
}
