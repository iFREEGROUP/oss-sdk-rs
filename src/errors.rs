//! Copyright The NoXF/oss-rust-sdk Authors
//! Copyright The iFREEGROUP/oss-sdk-rs Contributors

use super::model::error::Error as ErrorResponse;
use hmac::digest::InvalidLength;
use quick_xml::Error as QxmlError;
use reqwest::header::{InvalidHeaderValue as HttpInvalidHeaderValueError};
use reqwest::Error as ReqwestError;
use reqwest::{header::InvalidHeaderName as HttpInvalidHeaderNameError, StatusCode};
use serde::{Deserialize};
use std::io::Error as IoError;
use std::string::FromUtf8Error;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OSSError {
    #[error("object operation is not valid, status:{status_code:?}, message:{message:?}")]
    Object {
        status_code: StatusCode,
        message: String,
    },
    #[error("io error")]
    Io(#[from] IoError),
    #[error("string error")]
    String(#[from] FromUtf8Error),
    #[error("reqwest error")]
    Reqwest(#[from] ReqwestError),
    #[error("qxml error")]
    Qxml(#[from] QxmlError),
    #[error("parse xml error")]
    XmlParse(#[from] serde_xml_rs::Error),
    #[error("http error")]
    Http(#[from] HttpError),
    #[error("sign invalid length")]
    Sign(#[from] InvalidLength),
    #[error("unknown error")]
    Unknown,
}

#[derive(Debug, Error)]
pub enum HttpError {
    #[error("invalid head value")]
    HttpInvalidHeaderValue(#[from] HttpInvalidHeaderValueError),
    #[error("invalid head name")]
    HttpInvalidHeaderName(#[from] HttpInvalidHeaderNameError),
}

impl From<HttpInvalidHeaderValueError> for OSSError {
    fn from(e: HttpInvalidHeaderValueError) -> OSSError {
        OSSError::Http(HttpError::HttpInvalidHeaderValue(e))
    }
}

impl From<HttpInvalidHeaderNameError> for OSSError {
    fn from(e: HttpInvalidHeaderNameError) -> OSSError {
        OSSError::Http(HttpError::HttpInvalidHeaderName(e))
    }
}

pub fn status_to_response<'de, T>(status: StatusCode, text: String) -> Result<T, OSSError>
where
    T: Deserialize<'de> + Default,
{
    match status {
        StatusCode::OK
        | StatusCode::CREATED
        | StatusCode::ACCEPTED
        | StatusCode::NON_AUTHORITATIVE_INFORMATION
        | StatusCode::NO_CONTENT
        | StatusCode::RESET_CONTENT
        | StatusCode::PARTIAL_CONTENT
        | StatusCode::MULTI_STATUS
        | StatusCode::ALREADY_REPORTED => {
            let mut r = T::default();
            if !text.is_empty() {
                r = serde_xml_rs::from_str(&text)?;
                Ok(r)
            } else {
                Ok(r)
            }
        }
        StatusCode::BAD_REQUEST | StatusCode::FORBIDDEN | StatusCode::CONFLICT => {
            let er: ErrorResponse = serde_xml_rs::from_str(&text)?;
            Err(OSSError::Object {
                status_code: status,
                message: er.message,
            })
        }
        _ => Err(OSSError::Unknown),
    }
}
