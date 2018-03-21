//!
//! IPP protocol implementation for Rust
//!
//! Usage examples:
//!
//!```rust
//! // using raw API
//! use ipp::{IppRequestResponse, IppClient};
//! use ipp::consts::operation::Operation;
//!
//! let uri = "http://localhost:631/printers/test-printer";
//! let req = IppRequestResponse::new(Operation::GetPrinterAttributes, uri);
//! let client = IppClient::new(uri);
//! if let Ok(resp) = client.send_request(req) {
//!     if resp.header().operation_status <= 3 {
//!         println!("result: {:?}", resp.attributes());
//!     }
//! }
//!```
//!```rust
//! // using operation API
//! use ipp::{GetPrinterAttributes, IppClient};

//! let operation = GetPrinterAttributes::new();
//! let client = IppClient::new("http://localhost:631/printers/test-printer");
//! if let Ok(attrs) = client.send(operation) {
//!     for (_, v) in attrs.get_printer_attributes().unwrap() {
//!         println!("{}: {}", v.name(), v.value());
//!     }
//! }

//!```

extern crate byteorder;
extern crate clap;
extern crate reqwest;
extern crate url;
extern crate num_traits;
#[macro_use] extern crate enum_primitive_derive;
#[macro_use] extern crate log;

use std::result;
use std::fmt;
use std::io::{self, Read, Write};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

pub mod consts {
    //! This module holds IPP constants such as attribute names, operations and tags
    pub mod tag;
    pub mod statuscode;
    pub mod operation;
    pub mod attribute;
}

pub mod value;
pub mod parser;
pub mod request;
pub mod attribute;
pub mod client;
pub mod server;
pub mod operation;
pub mod util;
pub mod ffi;

pub use attribute::{IppAttribute, IppAttributeList};
pub use client::IppClient;
pub use operation::{IppOperation, PrintJob, GetPrinterAttributes, CreateJob, SendDocument};
pub use request::IppRequestResponse;
pub use value::IppValue;
pub const IPP_VERSION: u16 = 0x0101;

use consts::statuscode::StatusCode;

/// IPP value
#[derive(Debug)]
pub enum IppError {
    HttpError(reqwest::Error),
    IOError(::std::io::Error),
    RequestError(String),
    AttributeError(String),
    StatusError(consts::statuscode::StatusCode),
    TagError(u8),
    ParamError(clap::Error)
}

impl IppError {
    pub fn as_exit_code(&self) -> i32 {
        match self {
            &IppError::HttpError(_) => 2,
            &IppError::IOError(_) => 3,
            &IppError::RequestError(_) => 4,
            &IppError::AttributeError(_) => 5,
            &IppError::StatusError(_) => 6,
            &IppError::TagError(_) => 7,
            &IppError::ParamError(_) => 1
        }
    }
}

impl fmt::Display for IppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &IppError::HttpError(ref e) => write!(f, "{}", e),
            &IppError::IOError(ref e) => write!(f, "{}", e),
            &IppError::RequestError(ref e) => write!(f, "IPP request error: {}", e),
            &IppError::AttributeError(ref e) => write!(f, "IPP attribute error: {}", e),
            &IppError::StatusError(ref e) => write!(f, "IPP status error: {}", e),
            &IppError::TagError(ref e) => write!(f, "IPP tag error: {:0x}", e),
            &IppError::ParamError(ref e) => write!(f, "IPP tag error: {}", e)
        }
    }
}

impl From<io::Error> for IppError {
    fn from(error: io::Error) -> IppError {
        IppError::IOError(error)
    }
}

impl From<StatusCode> for IppError {
    fn from(code: StatusCode) -> IppError {
        IppError::StatusError(code)
    }
}

impl From<reqwest::Error> for IppError {
    fn from(error: reqwest::Error) -> IppError {
        IppError::HttpError(error)
    }
}

impl From<clap::Error> for IppError {
    fn from(error: clap::Error) -> IppError {
        IppError::ParamError(error)
    }
}

pub type Result<T> = result::Result<T, IppError>;

/// IPP request and response header
#[derive(Clone, Debug)]
pub struct IppHeader {
    pub version: u16,
    pub operation_status: u16,
    pub request_id: u32
}

impl IppHeader {
    pub fn from_reader(reader: &mut Read) -> Result<IppHeader> {
        let retval = IppHeader::new(
            reader.read_u16::<BigEndian>()?,
            reader.read_u16::<BigEndian>()?,
            reader.read_u32::<BigEndian>()?);
        Ok(retval)
    }

    /// Create IPP header
    pub fn new(version: u16, status: u16, request_id: u32) -> IppHeader {
        IppHeader { version, operation_status: status, request_id }
    }

    pub fn write(&self, writer: &mut Write) -> Result<usize> {
        writer.write_u16::<BigEndian>(self.version)?;
        writer.write_u16::<BigEndian>(self.operation_status)?;
        writer.write_u32::<BigEndian>(self.request_id)?;

        Ok(8)
    }
}

/// Trait which adds two methods to Read implementations: `read_string` and `read_vec`
pub trait ReadIppExt: Read {
    fn read_string(&mut self, len: usize) -> std::io::Result<String> {
        Ok(String::from_utf8_lossy(&self.read_vec(len)?).to_string())
    }

    fn read_vec(&mut self, len: usize) -> std::io::Result<Vec<u8>> {
        let mut namebuf: Vec<u8> = Vec::with_capacity(len);
        unsafe { namebuf.set_len(len) };

        self.read_exact(&mut namebuf)?;

        Ok(namebuf)
    }
}

impl<R: io::Read + ?Sized> ReadIppExt for R {}

#[test]
fn it_works() {
}
