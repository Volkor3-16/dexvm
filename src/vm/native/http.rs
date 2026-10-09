//! Shared HTTP types for okhttp and keiyoushi shims.
//! This module contains the common HTTP types and utilities that don't
//! depend on tachiyomi-specific features.

use crate::vm::error::JvmError;
use crate::vm::native::nat_fatal;
use crate::vm::native::NatErr;
use crate::vm::Vm;

/// A request built by the extension, handed to the host HTTP callback.
#[derive(Debug, Clone)]
pub struct HttpData {
    pub url: String,
    pub method: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
}

/// The host HTTP callback output; becomes an `okhttp3.Response` object.
#[derive(Debug, Clone)]
pub struct HttpResp {
    pub code: i32,
    pub message: String,
    pub headers: Vec<(String, String)>,
    /// Raw response payload; text bodies arrive as UTF-8 bytes.
    pub body: Option<Vec<u8>>,
}

impl HttpResp {
    pub fn ok(body: impl Into<String>) -> Self {
        HttpResp {
            code: 200,
            message: "OK".into(),
            headers: Vec::new(),
            body: Some(body.into().into_bytes()),
        }
    }
    pub fn ok_bytes(bytes: Vec<u8>) -> Self {
        HttpResp {
            code: 200,
            message: "OK".into(),
            headers: Vec::new(),
            body: Some(bytes),
        }
    }
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .rev()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }
}

pub(crate) fn check_network_url(vm: &mut Vm, url: &str) -> Result<(), NatErr> {
    let (scheme, rest) = url
        .split_once("://")
        .ok_or_else(|| nat_fatal(JvmError::Resolution(format!("invalid URL: {url}"))))?;
    let authority = rest
        .split(['/', '?', '#'])
        .next()
        .unwrap_or_default()
        .rsplit('@')
        .next()
        .unwrap_or_default();
    if authority.is_empty() {
        return Err(nat_fatal(JvmError::Resolution(format!(
            "invalid URL: {url}"
        ))));
    }
    let has_port = if authority.starts_with('[') {
        authority
            .find(']')
            .is_some_and(|end| authority.as_bytes().get(end + 1) == Some(&b':'))
    } else {
        authority.contains(':')
    };
    let default_port = match scheme {
        "http" => 80,
        "https" => 443,
        _ => return Err(nat_fatal(JvmError::Resolution(format!(
            "unsupported scheme: {scheme}"
        )))),
    };
    let port = if has_port {
        authority
            .split(':')
            .next_back()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(default_port)
    } else {
        default_port
    };
    let host = authority
        .split(':')
        .next()
        .unwrap_or(authority)
        .trim_start_matches('[')
        .trim_end_matches(']')
        .to_lowercase();
    vm.check_permission(&crate::permission::Permission::Network(
        crate::permission::NetworkPermission::Connect(format!("{host}:{port}")),
    )).map_err(nat_fatal)
}



/// Sync HTTP callback type (legacy): takes HttpData, returns HttpResp immediately.
pub type HttpCall = std::sync::Arc<dyn Fn(&HttpData) -> HttpResp + Send + Sync>;

/// Async HTTP callback: takes HttpData and a response handler for suspend/resume.
pub type HttpCallback = std::sync::Arc<dyn Fn(HttpData, Box<dyn FnOnce(HttpResp) + Send + 'static>) + Send + Sync>;

/// Per-host header resolver: given the lowercase request host, returns an
/// optional User-Agent and Cookie header value.
pub type HostHeaderFn = std::sync::Arc<dyn Fn(&str) -> (Option<String>, Option<String>) + Send + Sync>;
