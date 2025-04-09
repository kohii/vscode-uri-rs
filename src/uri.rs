/*
 * Rust implementation of vscode-uri
 * https://github.com/microsoft/vscode-uri
 */

use percent_encoding::{percent_decode_str, percent_encode, AsciiSet, CONTROLS};
use std::fmt;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use regex::Regex;
use lazy_static::lazy_static;
use crate::platform::is_windows;
use crate::char_code::CharCode;

lazy_static! {
    static ref SCHEME_PATTERN: Regex = Regex::new(r"^\w[\w\d+.-]*$").unwrap();
    static ref SINGLE_SLASH_START: Regex = Regex::new(r"^/").unwrap();
    static ref DOUBLE_SLASH_START: Regex = Regex::new(r"^//").unwrap();
    static ref URI_REGEX: Regex = Regex::new(r"^(([^:/?#]+?):)?(//([^/?#]*))?([^?#]*)(\?([^#]*))?(#(.*))?").unwrap();
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
        
        m.insert(CharCode::Space as u32, "%20");
        m
    };
}

fn validate_uri(uri: &URI, strict: bool) {
    if uri.scheme.is_empty() && strict {
        panic!("[UriError]: Scheme is missing: {{scheme: \"\", authority: \"{}\", path: \"{}\", query: \"{}\", fragment: \"{}\"}}",
            uri.authority, uri.path, uri.query, uri.fragment);
    }

    if !uri.scheme.is_empty() && !SCHEME_PATTERN.is_match(&uri.scheme) {
        panic!("[UriError]: Scheme contains illegal characters.");
    }

    if !uri.path.is_empty() {
        if !uri.authority.is_empty() && !SINGLE_SLASH_START.is_match(&uri.path) {
            panic!("[UriError]: If a URI contains an authority component, then the path component must either be empty or begin with a slash (\"/\") character");
        }
        if uri.authority.is_empty() && DOUBLE_SLASH_START.is_match(&uri.path) && uri.scheme != "file" {
            panic!("[UriError]: If a URI does not contain an authority component, then the path cannot begin with two slash characters (\"//\")");
        }
    }
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
    let mut native_encode_pos = -1;

    for (pos, c) in uri_component.char_indices() {
        let code = c as u32;

        if code == CharCode::Backslash as u32 && is_path && !is_windows() {
            if res.is_none() {
                res = Some(uri_component[0..pos].to_string());
            }
            if native_encode_pos != -1 {
                let encoded = percent_encode(uri_component[native_encode_pos as usize..pos].as_bytes(), CONTROLS).to_string();
                res = Some(res.unwrap_or_else(|| uri_component[0..native_encode_pos as usize].to_string()) + &encoded);
                native_encode_pos = -1;
            }
            res.as_mut().unwrap().push_str("%5C");
            continue;
        } else if code == CharCode::Colon as u32 && is_path && !is_authority {
            if res.is_none() {
                res = Some(uri_component[0..pos].to_string());
            }
            if native_encode_pos != -1 {
                let encoded = percent_encode(uri_component[native_encode_pos as usize..pos].as_bytes(), CONTROLS).to_string();
                res = Some(res.unwrap_or_else(|| uri_component[0..native_encode_pos as usize].to_string()) + &encoded);
                native_encode_pos = -1;
            }
            res.as_mut().unwrap().push_str("%3A");
            continue;
        }

        if (code >= CharCode::a as u32 && code <= CharCode::z as u32)
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
            if native_encode_pos != -1 {
                let encoded = percent_encode(uri_component[native_encode_pos as usize..pos].as_bytes(), CONTROLS).to_string();
                res = Some(res.unwrap_or_else(|| uri_component[0..native_encode_pos as usize].to_string()) + &encoded);
                native_encode_pos = -1;
            }
            if let Some(ref mut r) = res {
                r.push(c);
            }
        } else {
            if res.is_none() {
                res = Some(uri_component[0..pos].to_string());
            }

            if let Some(escaped) = ENCODE_TABLE.get(&code) {
                if native_encode_pos != -1 {
                    let encoded = percent_encode(uri_component[native_encode_pos as usize..pos].as_bytes(), CONTROLS).to_string();
                    res = Some(res.unwrap_or_else(|| uri_component[0..native_encode_pos as usize].to_string()) + &encoded);
                    native_encode_pos = -1;
                }

                res.as_mut().unwrap().push_str(escaped);
            } else if native_encode_pos == -1 {
                native_encode_pos = pos as i32;
            }
        }
    }

    if native_encode_pos != -1 {
        let encoded = percent_encode(uri_component[native_encode_pos as usize..].as_bytes(), CONTROLS).to_string();
        res = Some(res.unwrap_or_else(|| uri_component[0..native_encode_pos as usize].to_string()) + &encoded);
    }

    res.unwrap_or_else(|| uri_component.to_string())
}

fn encode_uri_component_minimal(path: &str) -> String {
    let mut res: Option<String> = None;
    
    for (pos, c) in path.char_indices() {
        let code = c as u32;
        
        if code == CharCode::Hash as u32 || code == CharCode::QuestionMark as u32 {
            if res.is_none() {
                res = Some(path[0..pos].to_string());
            }
            
            if let Some(escaped) = ENCODE_TABLE.get(&code) {
                res.as_mut().unwrap().push_str(escaped);
            }
        } else if let Some(ref mut r) = res {
            r.push(c);
        }
    }
    
    res.unwrap_or_else(|| path.to_string())
}

fn uri_to_fs_path(uri: &URI, keep_drive_letter_casing: bool) -> String {
    let mut value: String;
    
    if !uri.authority.is_empty() && uri.path.len() > 1 && uri.scheme == "file" {
        value = format!("//{}{}", uri.authority, uri.path);
    } else if uri.path.len() >= 3 
        && uri.path.chars().next() == Some('/') 
        && ((uri.path.chars().nth(1).unwrap() as u32 >= CharCode::A as u32 
            && uri.path.chars().nth(1).unwrap() as u32 <= CharCode::Z as u32) 
            || (uri.path.chars().nth(1).unwrap() as u32 >= CharCode::a as u32 
            && uri.path.chars().nth(1).unwrap() as u32 <= CharCode::z as u32))
        && uri.path.chars().nth(2) == Some(':') {
        
        if !keep_drive_letter_casing {
            let drive_letter = uri.path.chars().nth(1).unwrap().to_lowercase().next().unwrap();
            value = format!("{}:{}", drive_letter, &uri.path[3..]);
        } else {
            let drive_letter = uri.path.chars().nth(1).unwrap();
            value = format!("{}:{}", drive_letter, &uri.path[3..]);
        }
    } else if uri.path.len() >= 2 
        && ((uri.path.chars().next().unwrap() as u32 >= CharCode::A as u32 
            && uri.path.chars().next().unwrap() as u32 <= CharCode::Z as u32) 
            || (uri.path.chars().next().unwrap() as u32 >= CharCode::a as u32 
            && uri.path.chars().next().unwrap() as u32 <= CharCode::z as u32))
        && uri.path.chars().nth(1) == Some('/') {
        
        if !keep_drive_letter_casing {
            let drive_letter = uri.path.chars().next().unwrap().to_lowercase().next().unwrap();
            value = format!("{}:{}", drive_letter, &uri.path[1..]);
        } else {
            let drive_letter = uri.path.chars().next().unwrap();
            value = format!("{}:{}", drive_letter, &uri.path[1..]);
        }
    } else {
        value = uri.path.clone();
    }
    
    if value.contains('%') {
        value = percent_decode(&value);
    }
    
    if is_windows() {
        value = value.replace('/', "\\");
    }
    
    value
}

fn as_formatted(uri: &URI, skip_encoding: bool) -> String {
    let encoder = if !skip_encoding {
        |s: &str, is_path: bool, is_authority: bool| encode_uri_component_fast(s, is_path, is_authority)
    } else {
        |s: &str, _is_path: bool, _is_authority: bool| encode_uri_component_minimal(s)
    };
    
    let mut res = String::new();
    let scheme = &uri.scheme;
    let authority = &uri.authority;
    let path = &uri.path;
    let query = &uri.query;
    let fragment = &uri.fragment;
    
    if !scheme.is_empty() {
        res.push_str(&scheme.to_lowercase());
        res.push(':');
    }
    
    if !authority.is_empty() || scheme == "file" {
        res.push_str(SLASH);
        res.push_str(SLASH);
    }
    
    if !authority.is_empty() {
        let idx = authority.find('@');
        if let Some(idx) = idx {
            let userinfo = &authority[0..idx];
            let auth = &authority[idx + 1..];
            
            let user_idx = userinfo.rfind(':');
            if let Some(user_idx) = user_idx {
                res.push_str(&encoder(&userinfo[0..user_idx], false, false));
                res.push(':');
                res.push_str(&encoder(&userinfo[user_idx + 1..], false, true));
            } else {
                res.push_str(&encoder(userinfo, false, false));
            }
            
            res.push('@');
            
            let auth_lower = auth.to_lowercase();
            let auth_idx = auth_lower.rfind(':');
            
            if let Some(auth_idx) = auth_idx {
                res.push_str(&encoder(&auth_lower[0..auth_idx], false, true));
                res.push_str(&auth_lower[auth_idx..]);
            } else {
                res.push_str(&encoder(&auth_lower, false, true));
            }
        } else {
            let auth_lower = authority.to_lowercase();
            let auth_idx = auth_lower.rfind(':');
            
            if let Some(auth_idx) = auth_idx {
                res.push_str(&encoder(&auth_lower[0..auth_idx], false, true));
                res.push_str(&auth_lower[auth_idx..]);
            } else {
                res.push_str(&encoder(&auth_lower, false, true));
            }
        }
    }
    
    if !path.is_empty() {
        let mut path_to_use = path.clone();
        
        if path.len() >= 3 && path.starts_with('/') && path.chars().nth(2) == Some(':') {
            let code = path.chars().nth(1).unwrap() as u32;
            if code >= CharCode::A as u32 && code <= CharCode::Z as u32 {
                let drive_letter = ((code + 32) as u8) as char;
                path_to_use = format!("/{}:{}", drive_letter, &path[3..]);
            }
        } else if path.len() >= 2 && path.chars().nth(1) == Some(':') {
            let code = path.chars().next().unwrap() as u32;
            if code >= CharCode::A as u32 && code <= CharCode::Z as u32 {
                let drive_letter = ((code + 32) as u8) as char;
                path_to_use = format!("{}:{}", drive_letter, &path[2..]);
            }
        }
        
        res.push_str(&encoder(&path_to_use, true, false));
    }
    
    if !query.is_empty() {
        res.push('?');
        res.push_str(&encoder(query, false, false));
    }
    
    if !fragment.is_empty() {
        res.push('#');
        if !skip_encoding {
            res.push_str(&encode_uri_component_fast(fragment, false, false));
        } else {
            res.push_str(fragment);
        }
    }
    
    res
}

fn decode_uri_component_graceful(str: &str) -> String {
    match percent_decode_str(str).decode_utf8() {
        Ok(decoded) => decoded.to_string(),
        Err(_) => {
            if str.len() > 3 {
                format!("{}{}", &str[0..3], decode_uri_component_graceful(&str[3..]))
            } else {
                str.to_string()
            }
        }
    }
}

fn percent_decode(str: &str) -> String {
    if !ENCODED_AS_HEX.is_match(str) {
        return str.to_string();
    }
    
    ENCODED_AS_HEX.replace_all(str, |caps: &regex::Captures| {
        decode_uri_component_graceful(&caps[0])
    }).to_string()
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
    ) -> Self {
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
        validate_uri(&uri, false);
        uri
    }

    pub fn is_uri(thing: &dyn std::any::Any) -> bool {
        thing.is::<URI>()
    }

    pub fn parse(value: &str) -> Self {
        Self::parse_with_strict(value, false)
    }

    pub fn parse_with_strict(value: &str, strict: bool) -> Self {
        if value.is_empty() {
            return URI::new(EMPTY, EMPTY, EMPTY, EMPTY, EMPTY);
        }

        if let Some(captures) = URI_REGEX.captures(value) {
            let scheme = captures.get(2).map_or(EMPTY, |m| m.as_str());
            let authority = captures.get(4).map_or(EMPTY.to_string(), |m| percent_decode(m.as_str()));
            let path = captures.get(5).map_or(EMPTY.to_string(), |m| percent_decode(m.as_str()));
            let query = captures.get(7).map_or(EMPTY.to_string(), |m| percent_decode(m.as_str()));
            let fragment = captures.get(9).map_or(EMPTY.to_string(), |m| percent_decode(m.as_str()));
            
            return URI::new(
                scheme,
                authority,
                path,
                query,
                fragment
            );
        }
        
        URI::new(EMPTY, EMPTY, EMPTY, EMPTY, EMPTY)
    }

    pub fn file(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();
        let mut path_str = path.to_string_lossy().to_string();

        let mut authority = String::new();

        if is_windows() {
            path_str = path_str.replace('\\', SLASH);
        }

        if path_str.starts_with(SLASH) && path_str.chars().nth(1) == Some('/') {
            let idx = path_str[2..].find(SLASH).map(|i| i + 2).unwrap_or(path_str.len());
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
            let drive_letter = path_str.chars().next().unwrap().to_lowercase().next().unwrap();
            let rest = &path_str[2..];
            path_str = format!("/{}:{}", drive_letter, rest);
        }

        URI::new("file", authority, path_str, EMPTY, EMPTY)
    }

    pub fn from(components: &URIComponents) -> Self {
        URI::new(
            &components.scheme,
            &components.authority,
            &components.path,
            &components.query,
            &components.fragment,
        )
    }

    pub fn with(&self, change: URIChange) -> Self {
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
            return self.clone();
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
