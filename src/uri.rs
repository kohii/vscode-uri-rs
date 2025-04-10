/*
 * Rust implementation of vscode-uri
 * https://github.com/microsoft/vscode-uri
 */

use crate::char_code::CharCode;
use crate::platform::is_windows;
use lazy_static::lazy_static;
use percent_encoding::{percent_decode_str, percent_encode, CONTROLS};
use regex::Regex;
use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};

lazy_static! {
    static ref SCHEME_PATTERN: Regex = Regex::new(r"^[a-zA-Z][a-zA-Z0-9+.-]*$").unwrap();
    static ref SINGLE_SLASH_START: Regex = Regex::new(r"^/").unwrap();
    static ref DOUBLE_SLASH_START: Regex = Regex::new(r"^//").unwrap();
    static ref URI_REGEX: Regex =
        Regex::new(r"^(([^:/?#]+?):)?(//([^/?#]*))?([^?#]*)(\?([^#]*))?(#(.*))?").unwrap();
    static ref ENCODED_AS_HEX: Regex = Regex::new(r"(%[0-9A-Za-z][0-9A-Za-z])+").unwrap();
}

const EMPTY: &str = "";
const SLASH: &str = "/";
lazy_static! {
    static ref PATH_SEP_MARKER: Option<u8> = if is_windows() { Some(1) } else { None };
}

lazy_static! {
    static ref ENCODE_TABLE: HashMap<u32, &'static str> = {
        let mut m = HashMap::new();
        m.insert(CharCode::Colon as u32, "%3A");
        m.insert(CharCode::Slash as u32, "%2F");
        m.insert(CharCode::QuestionMark as u32, "%3F");
        m.insert(CharCode::Hash as u32, "%23");
        m.insert(CharCode::OpenSquareBracket as u32, "%5B");
        m.insert(CharCode::CloseSquareBracket as u32, "%5D");
        m.insert(CharCode::AtSign as u32, "%40");

        m.insert(CharCode::ExclamationMark as u32, "%21");
        m.insert(CharCode::DollarSign as u32, "%24");
        m.insert(CharCode::Ampersand as u32, "%26");
        m.insert(CharCode::SingleQuote as u32, "%27");
        m.insert(CharCode::OpenParen as u32, "%28");
        m.insert(CharCode::CloseParen as u32, "%29");
        m.insert(CharCode::Asterisk as u32, "%2A");
        m.insert(CharCode::Plus as u32, "%2B");
        m.insert(CharCode::Comma as u32, "%2C");
        m.insert(CharCode::Semicolon as u32, "%3B");
        m.insert(CharCode::Equals as u32, "%3D");
        m.insert(CharCode::PercentSign as u32, "%25");

        m.insert(CharCode::Space as u32, "%20");
        m
    };
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UriError {
    MissingScheme {
        scheme: String,
        authority: String,
        path: String,
        query: String,
        fragment: String,
    },
    IllegalSchemeCharacters,
    InvalidAuthorityPath,
    InvalidPathWithoutAuthority,
}

impl std::fmt::Display for UriError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UriError::MissingScheme { scheme, authority, path, query, fragment } => {
                write!(f, "Scheme is missing: {{scheme: \"{}\", authority: \"{}\", path: \"{}\", query: \"{}\", fragment: \"{}\"}}", 
                    scheme, authority, path, query, fragment)
            }
            UriError::IllegalSchemeCharacters => write!(f, "Scheme contains illegal characters"),
            UriError::InvalidAuthorityPath => write!(f, "If a URI contains an authority component, then the path component must either be empty or begin with a slash (\"/\") character"),
            UriError::InvalidPathWithoutAuthority => write!(f, "If a URI does not contain an authority component, then the path cannot begin with two slash characters (\"//\")"),
        }
    }
}

impl std::error::Error for UriError {}

fn validate_uri(uri: &URI, strict: bool) -> Result<(), UriError> {
    if uri.scheme.is_empty() && strict {
        return Err(UriError::MissingScheme {
            scheme: uri.scheme.clone(),
            authority: uri.authority.clone(),
            path: uri.path.clone(),
            query: uri.query.clone(),
            fragment: uri.fragment.clone(),
        });
    }

    if !uri.scheme.is_empty() && !SCHEME_PATTERN.is_match(&uri.scheme.to_lowercase()) {
        return Err(UriError::IllegalSchemeCharacters);
    }

    if !uri.path.is_empty() {
        if !uri.authority.is_empty() && !SINGLE_SLASH_START.is_match(&uri.path) {
            return Err(UriError::InvalidAuthorityPath);
        }
        if uri.authority.is_empty() && DOUBLE_SLASH_START.is_match(&uri.path) {
            return Err(UriError::InvalidPathWithoutAuthority);
        }
    }
    Ok(())
}

fn scheme_fix(scheme: &str, strict: bool) -> String {
    if scheme.is_empty() && !strict {
        return "file".to_string();
    }
    scheme.to_string()
}

fn reference_resolution(scheme: &str, path: &str) -> String {
    match scheme {
        "https" | "http" | "file" => {
            if path.is_empty() {
                SLASH.to_string()
            } else if !path.starts_with(SLASH) {
                format!("{}{}", SLASH, path)
            } else {
                path.to_string()
            }
        }
        _ => path.to_string(),
    }
}

fn encode_uri_component_fast(uri_component: &str, is_path: bool, is_authority: bool) -> String {
    let mut res: Option<String> = None;
    let mut native_encode_pos: i32 = -1;

    for (i, c) in uri_component.chars().enumerate() {
        let code = c as u32;

        // unreserved characters: https://tools.ietf.org/html/rfc3986#section-2.3
        if (code >= CharCode::SmallA as u32 && code <= CharCode::SmallZ as u32)
            || (code >= CharCode::A as u32 && code <= CharCode::Z as u32)
            || (code >= CharCode::Digit0 as u32 && code <= CharCode::Digit9 as u32)
            || code == CharCode::Dash as u32
            || code == CharCode::Period as u32
            || code == CharCode::Underline as u32
            || code == CharCode::Tilde as u32
            || (is_path && code == CharCode::Slash as u32)
            || (is_authority && code == CharCode::OpenSquareBracket as u32)
            || (is_authority && code == CharCode::CloseSquareBracket as u32)
            || (is_authority && code == CharCode::Colon as u32)
        {
            // check if we are delaying native encode
            if native_encode_pos != -1 {
                let encoded = percent_encode(
                    uri_component[native_encode_pos as usize..i].as_bytes(),
                    CONTROLS,
                )
                .to_string()
                .to_uppercase();
                res = Some(
                    res.unwrap_or_else(|| uri_component[0..native_encode_pos as usize].to_string())
                        + &encoded,
                );
                native_encode_pos = -1;
            }
            // check if we write into a new string
            if let Some(ref mut r) = res {
                r.push(c);
            }
        } else {
            // encoding needed, we need to allocate a new string
            if res.is_none() {
                res = Some(uri_component[0..i].to_string());
            }

            // check with default table first
            let escaped = if code == CharCode::Backslash as u32 && is_path {
                Some("%5C")
            } else {
                ENCODE_TABLE.get(&code).copied()
            };

            if let Some(escaped) = escaped {
                // check if we are delaying native encode
                if native_encode_pos != -1 {
                    let encoded = percent_encode(
                        uri_component[native_encode_pos as usize..i].as_bytes(),
                        CONTROLS,
                    )
                    .to_string()
                    .to_uppercase();
                    res = Some(
                        res.unwrap_or_else(|| {
                            uri_component[0..native_encode_pos as usize].to_string()
                        }) + &encoded,
                    );
                    native_encode_pos = -1;
                }

                // append escaped variant to result
                res.as_mut().unwrap().push_str(escaped);
            } else if code > 127 {
                // Always encode non-ASCII characters
                let bytes = c.to_string().as_bytes().to_vec();
                let encoded = bytes
                    .iter()
                    .map(|b| format!("%{:02X}", b))
                    .collect::<String>();
                res.as_mut().unwrap().push_str(&encoded);
            } else if native_encode_pos == -1 {
                // use native encode only when needed
                native_encode_pos = i as i32;
            }
        }
    }

    if native_encode_pos != -1 {
        let encoded = percent_encode(
            uri_component[native_encode_pos as usize..].as_bytes(),
            CONTROLS,
        )
        .to_string()
        .to_uppercase();
        res = Some(
            res.unwrap_or_else(|| uri_component[0..native_encode_pos as usize].to_string())
                + &encoded,
        );
    }

    res.unwrap_or_else(|| uri_component.to_string())
}

fn encode_uri_component_minimal(uri_component: &str) -> String {
    let mut res = String::new();
    for c in uri_component.chars() {
        let code = c as u32;
        if code == CharCode::Hash as u32 || code == CharCode::QuestionMark as u32 {
            res.push('%');
            res.push_str(&format!("{:02X}", code));
        } else {
            res.push(c);
        }
    }
    res
}

fn uri_to_fs_path(uri: &URI, keep_drive_letter_casing: bool) -> String {
    let mut value: String;

    if !uri.authority.is_empty() && uri.path.len() > 1 && uri.scheme == "file" {
        value = format!("//{}{}", uri.authority, uri.path);
    } else if uri.path.chars().next() == Some('/')
        && ((uri
            .path
            .chars()
            .nth(1)
            .map_or(false, |c| c.is_ascii_uppercase())
            && uri.path.chars().nth(2) == Some(':'))
            || (uri
                .path
                .chars()
                .nth(1)
                .map_or(false, |c| c.is_ascii_lowercase())
                && uri.path.chars().nth(2) == Some(':')))
    {
        if !keep_drive_letter_casing {
            value = format!(
                "{}{}",
                uri.path
                    .chars()
                    .nth(1)
                    .unwrap()
                    .to_lowercase()
                    .next()
                    .unwrap(),
                uri.path.chars().skip(2).collect::<String>()
            );
        } else {
            value = uri.path.chars().skip(1).collect();
        }
    } else {
        value = uri.path.clone();
    }

    value = percent_decode(&value).to_string();

    if is_windows() {
        value = value.replace('/', "\\");
    }

    value
}

fn as_formatted(uri: &URI, skip_encoding: bool) -> String {
    let mut res = String::new();

    let encoder = if !skip_encoding {
        |s: &str, is_path: bool, is_authority: bool| {
            if is_path && is_windows() {
                encode_uri_component_fast(&s.replace('\\', "%5C"), is_path, is_authority)
            } else {
                encode_uri_component_fast(s, is_path, is_authority)
            }
        }
    } else {
        |s: &str, _is_path: bool, _is_authority: bool| encode_uri_component_minimal(s)
    };

    if !uri.scheme.is_empty() {
        res.push_str(&uri.scheme);
        res.push(':');
    }

    if !uri.authority.is_empty() || uri.scheme == "file" {
        res.push_str("//");
    }

    if !uri.authority.is_empty() {
        let mut authority = uri.authority.clone();
        if let Some(idx) = authority.find('@') {
            let userinfo = authority[..idx].to_string();
            authority = authority[idx + 1..].to_string();
            if let Some(idx) = userinfo.rfind(':') {
                res.push_str(&encoder(&userinfo[..idx], false, false));
                res.push(':');
                res.push_str(&encoder(&userinfo[idx + 1..], false, true));
            } else {
                res.push_str(&encoder(&userinfo, false, false));
            }
            res.push('@');
            authority = authority.to_lowercase();
            if let Some(idx) = authority.rfind(':') {
                res.push_str(&encoder(&authority[..idx], false, true));
                res.push_str(&authority[idx..]);
            } else {
                res.push_str(&encoder(&authority, false, true));
            }
        } else {
            authority = authority.to_lowercase();
            if let Some(idx) = authority.rfind(':') {
                res.push_str(&encoder(&authority[..idx], false, true));
                res.push_str(&authority[idx..]);
            } else {
                res.push_str(&encoder(&authority, false, true));
            }
        }
    }

    if !uri.path.is_empty() {
        let mut path = uri.path.clone();
        if path.len() >= 3 {
            let mut chars = path.chars();
            if let (Some('/'), Some(second), Some(':')) = (chars.next(), chars.next(), chars.next())
            {
                let code = second as u32;
                if code >= CharCode::A as u32 && code <= CharCode::Z as u32 {
                    let drive_letter = ((code + 32) as u8) as char;
                    path = format!("/{}{}", drive_letter, &path[2..]);
                }
            }
        } else if path.len() >= 2 {
            let mut chars = path.chars();
            if let (Some(first), Some(':')) = (chars.next(), chars.next()) {
                let code = first as u32;
                if code >= CharCode::A as u32 && code <= CharCode::Z as u32 {
                    let drive_letter = ((code + 32) as u8) as char;
                    path = format!("{}{}", drive_letter, &path[1..]);
                }
            }
        }
        res.push_str(&encoder(&path, true, false));
    }

    if !uri.query.is_empty() {
        res.push('?');
        if skip_encoding {
            let mut query = String::new();
            for c in uri.query.chars() {
                let code = c as u32;
                if code == CharCode::Hash as u32 {
                    query.push_str("%23");
                } else {
                    query.push(c);
                }
            }
            res.push_str(&query);
        } else {
            res.push_str(&encoder(&uri.query, false, false));
        }
    }

    if !uri.fragment.is_empty() {
        res.push('#');
        if !skip_encoding {
            res.push_str(&encode_uri_component_fast(&uri.fragment, false, false));
        } else {
            res.push_str(&uri.fragment);
        }
    }

    res
}

fn decode_uri_component_graceful(str: &str) -> String {
    if str.len() < 3 {
        return str.to_string();
    }

    match percent_decode_str(str).decode_utf8() {
        Ok(decoded) => decoded.to_string(),
        Err(_) => {
            format!("{}{}", &str[0..3], decode_uri_component_graceful(&str[3..]))
        }
    }
}

fn percent_decode(str: &str) -> String {
    if !ENCODED_AS_HEX.is_match(str) {
        return str.to_string();
    }

    let mut result = String::new();
    let mut last_end = 0;
    for caps in ENCODED_AS_HEX.find_iter(str) {
        result.push_str(&str[last_end..caps.start()]);
        result.push_str(&decode_uri_component_graceful(caps.as_str()));
        last_end = caps.end();
    }
    result.push_str(&str[last_end..]);
    result
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct URI {
    scheme: String,
    authority: String,
    path: String,
    query: String,
    fragment: String,
}

impl URI {
    pub fn new(
        scheme: impl Into<String>,
        authority: impl Into<String>,
        path: impl Into<String>,
        query: impl Into<String>,
        fragment: impl Into<String>,
    ) -> Result<Self, UriError> {
        let scheme = scheme.into();
        let authority = authority.into();
        let path = path.into();
        let query = query.into();
        let fragment = fragment.into();

        let scheme = scheme_fix(&scheme, false);
        let path = reference_resolution(&scheme, &path);

        let uri = URI {
            scheme,
            authority,
            path,
            query,
            fragment,
        };
        validate_uri(&uri, false)?;
        Ok(uri)
    }

    pub fn is_uri(thing: &dyn std::any::Any) -> bool {
        thing.is::<URI>()
    }

    pub fn parse(value: &str) -> Result<Self, UriError> {
        Self::parse_with_strict(value, false)
    }

    pub fn parse_with_strict(value: &str, _strict: bool) -> Result<Self, UriError> {
        if value.is_empty() {
            return URI::new(EMPTY, EMPTY, EMPTY, EMPTY, EMPTY);
        }

        if let Some(captures) = URI_REGEX.captures(value) {
            let scheme = captures.get(2).map_or(EMPTY, |m| m.as_str());
            let authority = captures
                .get(4)
                .map_or(EMPTY.to_string(), |m| percent_decode(m.as_str()));
            let path = captures
                .get(5)
                .map_or(EMPTY.to_string(), |m| percent_decode(m.as_str()));
            let query = captures
                .get(7)
                .map_or(EMPTY.to_string(), |m| percent_decode(m.as_str()));
            let fragment = captures
                .get(9)
                .map_or(EMPTY.to_string(), |m| percent_decode(m.as_str()));

            return URI::new(scheme, authority, path, query, fragment);
        }

        URI::new(EMPTY, EMPTY, EMPTY, EMPTY, EMPTY)
    }

    pub fn file(path: impl AsRef<Path>) -> Result<Self, UriError> {
        let path = path.as_ref();
        let mut path_str = path.to_string_lossy().to_string();

        let mut authority = String::new();

        if is_windows() {
            path_str = path_str.replace('\\', SLASH);
        }

        if path_str.starts_with(SLASH) && path_str.chars().nth(1) == Some('/') {
            let idx = path_str[2..]
                .find(SLASH)
                .map(|i| i + 2)
                .unwrap_or(path_str.len());
            if idx == 2 {
                authority = path_str[2..].to_string();
                path_str = SLASH.to_string();
            } else {
                authority = path_str[2..idx].to_string();
                path_str = if idx < path_str.len() {
                    path_str[idx..].to_string()
                } else {
                    SLASH.to_string()
                };
            }
        }

        if path_str.len() >= 2 && path_str.chars().nth(1) == Some(':') {
            let drive_letter = path_str
                .chars()
                .next()
                .unwrap()
                .to_lowercase()
                .next()
                .unwrap();
            let rest = &path_str[2..];
            path_str = format!("/{}:{}", drive_letter, rest);
        }

        URI::new("file", authority, path_str, EMPTY, EMPTY)
    }

    pub fn from(components: &URIComponents) -> Result<Self, UriError> {
        URI::new(
            &components.scheme,
            &components.authority,
            &components.path,
            &components.query,
            &components.fragment,
        )
    }

    pub fn with(&self, change: URIChange) -> Result<Self, UriError> {
        let scheme = change.scheme.unwrap_or_else(|| self.scheme.clone());
        let authority = change.authority.unwrap_or_else(|| self.authority.clone());
        let path = change.path.unwrap_or_else(|| self.path.clone());
        let query = change.query.unwrap_or_else(|| self.query.clone());
        let fragment = change.fragment.unwrap_or_else(|| self.fragment.clone());

        if scheme == self.scheme
            && authority == self.authority
            && path == self.path
            && query == self.query
            && fragment == self.fragment
        {
            return Ok(self.clone());
        }

        URI::new(scheme, authority, path, query, fragment)
    }

    pub fn scheme(&self) -> &str {
        &self.scheme
    }

    pub fn authority(&self) -> &str {
        &self.authority
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn fragment(&self) -> &str {
        &self.fragment
    }

    pub fn fs_path(&self) -> PathBuf {
        PathBuf::from(uri_to_fs_path(self, false))
    }

    pub fn to_string(&self, skip_encoding: bool) -> String {
        as_formatted(self, skip_encoding)
    }
}

impl fmt::Display for URI {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string(false))
    }
}

#[derive(Debug, Default, Clone)]
pub struct URIChange {
    pub scheme: Option<String>,
    pub authority: Option<String>,
    pub path: Option<String>,
    pub query: Option<String>,
    pub fragment: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct URIComponents {
    pub scheme: String,
    pub authority: String,
    pub path: String,
    pub query: String,
    pub fragment: String,
}
