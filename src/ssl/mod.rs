/*
 *   __  __                 _     _       _
 *  |  \/  | ___  ___  __ _| |   (_)_ __ | | __
 *  | |\/| |/ _ \/ __|/ _` | |   | | '_ \| |/ /
 *  | |  | |  __/\__ \ (_| | |___| | | | |   <
 *  |_|  |_|\___||___/\__,_|_____|_|_| |_|_|\_\
 *
 * Copyright (c) 2017-2018, The MesaLink Authors.
 * All rights reserved.
 *
 * This work is licensed under the terms of the BSD 3-Clause License.
 * For a copy, see the LICENSE file.
 *
 */

use ring::rand;
use ring::rand::SecureRandom;

#[doc(hidden)]
pub const MAGIC_SIZE: usize = 4;

lazy_static! {
    #[doc(hidden)]
    pub static ref MAGIC: [u8; MAGIC_SIZE] = {
        let mut number = [0u8; MAGIC_SIZE];
        if rand::SystemRandom::new().fill(&mut number).is_ok() {
            let number = number;
            number
        } else {
            panic!("Getrandom error");
        }
    };
}

#[doc(hidden)]
pub trait MesalinkOpaquePointerType {
    fn check_magic(&self) -> bool;
}

/// Implementations of OpenSSL ERR APIs.
/// Please also refer to the header file at mesalink/openssl/err.h
#[macro_use]
pub mod err;

#[macro_use]
mod macros {
    #[macro_export]
    macro_rules! error {
        ($code:expr) => {{
            use ssl::err::MesalinkError;
            MesalinkError::new($code, call_site!())
        }};
    }

    // A utility macro that wraps each inner API implementation and checks its
    // returned value. This macro also catches panics and prevents unwinding across
    // FFI boundaries. Note that the panic mode must be set to `unwind` in
    // Cargo.toml.
    #[macro_export]
    macro_rules! check_inner_result {
        ($inner:expr, $err_ret:expr) => {{
            use ssl::err::{ErrorCode, ErrorQueue};
            use std::panic;
            match panic::catch_unwind(panic::AssertUnwindSafe(|| $inner))
                .unwrap_or_else(|_| Err(error!(ErrorCode::MesalinkErrorPanic)))
            {
                Ok(r) => r,
                Err(e) => {
                    ErrorQueue::push_error(e);
                    $err_ret
                }
            }
        }};
    }
}

/// Implementations of OpenSSL SSL APIs.
/// Please also refer to the header file at mesalink/openssl/ssl.h
pub mod ssl;

/// Implementations of OpenSSL X509 APIs.
/// Please also refer to the header file at mesalink/openssl/x509.h
pub mod x509;

/// Implementations of OpenSSL STACK APIs.
/// Please also refer to the header file at mesalink/openssl/safestack.h
pub mod safestack;

#[doc(hidden)]
#[repr(C)]
pub enum SslConstants {
    SslError = -1,
    SslFailure = 0,
    SslSuccess = 1,
}

#[doc(hidden)]
#[repr(C)]
#[derive(Clone)]
pub enum SslSessionCacheModes {
    SslSessCacheOff = 0x0,
    SslSessCacheClient = 0x1,
    SslSessCacheServer = 0x2,
    SslSessCacheBoth = 0x3,
}

#[doc(hidden)]
#[repr(C)]
#[derive(Clone)]
pub enum SslVerifyModes {
    SslVerifyNone = 0,
    SslVerifyPeer = 0x1,
    SslVerifyFailIfNoPeerCert = 0x2,
}

use libc::c_int;
pub const SSL_ERROR: c_int = SslConstants::SslError as c_int;
pub const SSL_FAILURE: c_int = SslConstants::SslFailure as c_int;
pub const SSL_SUCCESS: c_int = SslConstants::SslSuccess as c_int;

#[macro_use]
#[doc(hidden)]
pub mod error_san {
    use ssl::MesalinkOpaquePointerType;
    use ssl::err::{ErrorCode, MesalinkInnerResult};

    pub fn sanitize_const_ptr_for_ref<'a, T>(ptr: *const T) -> MesalinkInnerResult<&'a T>
    where
        T: MesalinkOpaquePointerType,
    {
        let ptr = ptr as *mut T;
        sanitize_ptr_for_mut_ref(ptr).map(|r| r as &'a T)
    }

    pub fn sanitize_ptr_for_ref<'a, T>(ptr: *mut T) -> MesalinkInnerResult<&'a T>
    where
        T: MesalinkOpaquePointerType,
    {
        sanitize_ptr_for_mut_ref(ptr).map(|r| r as &'a T)
    }

    pub fn sanitize_ptr_for_mut_ref<'a, T>(ptr: *mut T) -> MesalinkInnerResult<&'a mut T>
    where
        T: MesalinkOpaquePointerType,
    {
        if ptr.is_null() {
            return Err(error!(ErrorCode::MesalinkErrorNullPointer));
        }
        let obj_ref: &mut T = unsafe { &mut *ptr };
        match obj_ref.check_magic() {
            true => Ok(obj_ref),
            false => Err(error!(ErrorCode::MesalinkErrorMalformedObject)),
        }
    }
}
