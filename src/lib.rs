use percent_encoding::{percent_decode_str, percent_encode, AsciiSet, CONTROLS};
use std::fmt;
use std::path::{Path, PathBuf};
use url::Url;

const URI_ENCODING_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'<')
    .add(b'>')
    .add(b'`')
    .add(b'?')
    .add(b'{')
    .add(b'}')
    .add(b'/')
    .add(b':')
    .add(b';')
    .add(b'=')
    .add(b'@')
    .add(b'[')
    .add(b'\\')
    .add(b']')
    .add(b'^')
    .add(b'|')
    .add(b'$')
    .add(b'&')
    .add(b'+')
    .add(b',')
    .add(b'\'');

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

        let path = match scheme.as_str() {
            "https" | "http" | "file" => {
                if path.is_empty() {
                    "/".to_string()
                } else if !path.starts_with('/') {
                    format!("/{}", path)
                } else {
                    path
                }
            }
            _ => path,
        };

        let uri = URI {
            scheme,
            authority,
            path,
            query,
            fragment,
        };
        uri.validate();
        uri
    }

    fn validate(&self) {
        if !self.scheme.is_empty() {
            let valid_scheme = self.scheme.chars().enumerate().all(|(i, c)| {
                if i == 0 {
                    c.is_ascii_alphabetic()
                } else {
                    c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.'
                }
            });
            if !valid_scheme {
                panic!("URI Error: Scheme contains illegal characters");
            }
        }

        if !self.path.is_empty() {
            if !self.authority.is_empty() && !self.path.starts_with('/') {
                panic!("URI Error: If a URI contains an authority component, then the path component must either be empty or begin with a slash (\"/\") character");
            }
            if self.authority.is_empty() && self.path.starts_with("//") {
                panic!("URI Error: If a URI does not contain an authority component, then the path cannot begin with two slash characters (\"//\")");
            }
        }
    }

    pub fn parse(value: &str) -> Self {
        if value.is_empty() {
            return URI::new("", "", "", "", "");
        }

        match Url::parse(value) {
            Ok(url) => {
                let scheme = url.scheme().to_string();
                let authority = url.host_str().unwrap_or("").to_string();
                let authority = if let Some(port) = url.port() {
                    format!("{}:{}", authority, port)
                } else {
                    authority
                };
                let username = url.username();
                let authority = if !username.is_empty() {
                    if let Some(password) = url.password() {
                        if !password.is_empty() {
                            format!("{}:{}@{}", username, password, authority)
                        } else {
                            format!("{}@{}", username, authority)
                        }
                    } else {
                        format!("{}@{}", username, authority)
                    }
                } else {
                    authority
                };
                
                let mut path = url.path().to_string();
                path = percent_decode_str(&path).decode_utf8_lossy().to_string();
                
                let query = url.query().unwrap_or("").to_string();
                let fragment = url.fragment().unwrap_or("").to_string();
                
                URI::new(scheme, authority, path, query, fragment)
            }
            Err(_) => {
                if value.starts_with("file:") {
                    let path = value.trim_start_matches("file://").trim_start_matches("file:");
                    let decoded_path = percent_decode_str(path).decode_utf8_lossy().to_string();
                    URI::file(decoded_path)
                } else {
                    let mut scheme = String::new();
                    let mut authority = String::new();
                    let mut path = String::new();
                    let mut query = String::new();
                    let mut fragment = String::new();

                    if let Some(scheme_end) = value.find(':') {
                        scheme = value[..scheme_end].to_string();
                        let rest = &value[scheme_end + 1..];

                        if rest.starts_with("//") {
                            let auth_path = &rest[2..];
                            if let Some(auth_end) = auth_path.find('/') {
                                authority = auth_path[..auth_end].to_string();
                                let path_query_frag = &auth_path[auth_end..];

                                if let Some(query_start) = path_query_frag.find('?') {
                                    path = path_query_frag[..query_start].to_string();
                                    let query_frag = &path_query_frag[query_start + 1..];

                                    if let Some(frag_start) = query_frag.find('#') {
                                        query = query_frag[..frag_start].to_string();
                                        fragment = query_frag[frag_start + 1..].to_string();
                                    } else {
                                        query = query_frag.to_string();
                                    }
                                } else if let Some(frag_start) = path_query_frag.find('#') {
                                    path = path_query_frag[..frag_start].to_string();
                                    fragment = path_query_frag[frag_start + 1..].to_string();
                                } else {
                                    path = path_query_frag.to_string();
                                }
                            } else {
                                authority = auth_path.to_string();
                            }
                        } else {
                            let path_query_frag = rest;
                            if let Some(query_start) = path_query_frag.find('?') {
                                path = path_query_frag[..query_start].to_string();
                                let query_frag = &path_query_frag[query_start + 1..];

                                if let Some(frag_start) = query_frag.find('#') {
                                    query = query_frag[..frag_start].to_string();
                                    fragment = query_frag[frag_start + 1..].to_string();
                                } else {
                                    query = query_frag.to_string();
                                }
                            } else if let Some(frag_start) = path_query_frag.find('#') {
                                path = path_query_frag[..frag_start].to_string();
                                fragment = path_query_frag[frag_start + 1..].to_string();
                            } else {
                                path = path_query_frag.to_string();
                            }
                        }
                    } else {
                        path = value.to_string();
                    }

                    path = percent_decode_str(&path).decode_utf8_lossy().to_string();
                    query = percent_decode_str(&query).decode_utf8_lossy().to_string();
                    fragment = percent_decode_str(&fragment).decode_utf8_lossy().to_string();

                    URI::new(scheme, authority, path, query, fragment)
                }
            }
        }
    }

    pub fn file(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();
        let path_str = path.to_string_lossy();

        let mut authority = String::new();
        let mut uri_path = String::new();

        if cfg!(windows) {
            let path_str = path_str.replace('\\', "/");
            if path_str.starts_with("//") {
                let parts: Vec<&str> = path_str[2..].splitn(2, '/').collect();
                if parts.len() > 0 {
                    authority = parts[0].to_string();
                    if parts.len() > 1 {
                        uri_path = format!("/{}", parts[1]);
                    } else {
                        uri_path = "/".to_string();
                    }
                }
            } else {
                uri_path = path_str;
                if uri_path.len() >= 2 && uri_path.chars().nth(1) == Some(':') {
                    let drive = uri_path.chars().next().unwrap().to_lowercase().to_string();
                    uri_path = format!("{}{}", drive, &uri_path[1..]);
                }
                if !uri_path.starts_with('/') {
                    uri_path = format!("/{}", uri_path);
                }
            }
        } else {
            uri_path = path_str.to_string();
            if !uri_path.starts_with('/') {
                uri_path = format!("/{}", uri_path);
            }
        }

        URI::new("file", authority, uri_path, "", "")
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
        if self.scheme != "file" {
            eprintln!("Warning: calling fs_path with scheme {}", self.scheme);
        }

        let mut path = self.path.clone();

        if cfg!(windows) {
            if !self.authority.is_empty() {
                path = format!("\\\\{}\\{}", self.authority, path.trim_start_matches('/').replace('/', "\\"));
            } else {
                if path.len() >= 3 && path.starts_with('/') && path.chars().nth(2) == Some(':') {
                    path = path[1..].to_string();
                }
                
                path = path.replace('/', "\\");
                if path.starts_with('\\') && !path.starts_with("\\\\") {
                    path = path[1..].to_string();
                }
            }
        } else if !self.authority.is_empty() {
            path = format!("//{}{}", self.authority, path);
        }

        let decoded_path = percent_decode_str(&path).decode_utf8_lossy().to_string();
        PathBuf::from(decoded_path)
    }

    pub fn to_string(&self, skip_encoding: bool) -> String {
        let mut result = String::new();

        if !self.scheme.is_empty() {
            result.push_str(&self.scheme.to_lowercase());
            result.push(':');
        }

        if !self.authority.is_empty() {
            result.push_str("//");
            if skip_encoding {
                result.push_str(&self.authority);
            } else {
                let encoded = percent_encode(self.authority.as_bytes(), URI_ENCODING_SET)
                    .to_string()
                    .replace("%3A", ":")
                    .replace("%40", "@");
                result.push_str(&encoded);
            }
        } else if self.scheme == "file" {
            result.push_str("//");
        }

        if skip_encoding {
            result.push_str(&self.path);
        } else {
            let path = if self.path.starts_with('/') {
                &self.path[1..]
            } else {
                &self.path
            };
            
            if self.path.starts_with('/') || self.path.is_empty() {
                result.push('/');
            }
            
            let segments: Vec<&str> = path.split('/').collect();
            for (i, segment) in segments.iter().enumerate() {
                if i > 0 {
                    result.push('/');
                }
                if !segment.is_empty() {
                    let encoded = percent_encode(segment.as_bytes(), URI_ENCODING_SET).to_string();
                    result.push_str(&encoded);
                }
            }
            
            if self.path.ends_with('/') && !result.ends_with('/') && !self.path.is_empty() {
                result.push('/');
            }
        }

        if !self.query.is_empty() {
            result.push('?');
            if skip_encoding {
                result.push_str(&self.query);
            } else {
                let encoded = percent_encode(self.query.as_bytes(), URI_ENCODING_SET).to_string();
                result.push_str(&encoded);
            }
        }

        if !self.fragment.is_empty() {
            result.push('#');
            if skip_encoding {
                result.push_str(&self.fragment);
            } else {
                let encoded = percent_encode(self.fragment.as_bytes(), URI_ENCODING_SET).to_string();
                result.push_str(&encoded);
            }
        }

        result
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

pub mod utils {
    use super::URI;

    pub fn join_path(uri: &URI, paths: &[&str]) -> URI {
        let mut result = uri.path().to_string();
        
        for path in paths {
            if result.ends_with('/') {
                result.push_str(path);
            } else {
                result.push('/');
                result.push_str(path);
            }
        }
        
        result = normalize_path(&result);
        
        uri.with(super::URIChange {
            path: Some(result),
            ..Default::default()
        })
    }

    pub fn resolve_path(uri: &URI, paths: &[&str]) -> URI {
        let mut base = uri.path().to_string();
        if !base.starts_with('/') {
            base = format!("/{}", base);
        }
        
        let mut result = base;
        for path in paths {
            if path.starts_with('/') {
                result = path.to_string();
            } else {
                let mut segments: Vec<&str> = result.split('/').collect();
                let path_segments: Vec<&str> = path.split('/').collect();
                
                for segment in path_segments {
                    if segment == "." {
                        continue;
                    } else if segment == ".." {
                        if segments.len() > 1 {
                            segments.pop();
                        }
                    } else {
                        segments.push(segment);
                    }
                }
                
                result = segments.join("/");
                if result.is_empty() {
                    result = "/".to_string();
                }
            }
        }
        
        if !uri.path().starts_with('/') && uri.authority().is_empty() && result.starts_with('/') {
            result = result[1..].to_string();
        }
        
        uri.with(super::URIChange {
            path: Some(result),
            ..Default::default()
        })
    }

    pub fn dirname(uri: &URI) -> URI {
        if uri.path().is_empty() || uri.path() == "/" {
            return uri.clone();
        }
        
        let path = uri.path();
        let mut result = path.to_string();
        
        if result.ends_with('/') && result != "/" {
            result.pop();
        }
        
        if let Some(last_slash) = result.rfind('/') {
            if last_slash == 0 {
                result = "/".to_string();
            } else {
                result = result[..last_slash].to_string();
            }
        } else {
            result = ".".to_string();
        }
        
        if result == "." && uri.scheme() != "file" {
            result = String::new();
        }
        
        uri.with(super::URIChange {
            path: Some(result),
            ..Default::default()
        })
    }

    pub fn basename(uri: &URI) -> String {
        let path = uri.path();
        if path.is_empty() {
            return String::new();
        }
        
        let mut path_str = path.to_string();
        if path_str.ends_with('/') && path_str != "/" {
            path_str.pop();
        }
        
        if let Some(last_slash) = path_str.rfind('/') {
            path_str[last_slash + 1..].to_string()
        } else {
            path_str
        }
    }

    pub fn extname(uri: &URI) -> String {
        let base = self::basename(uri);
        if base.is_empty() {
            return String::new();
        }
        
        if let Some(dot_pos) = base.rfind('.') {
            if dot_pos > 0 {  // Ensure it's not a hidden file (starting with .)
                return base[dot_pos..].to_string();
            }
        }
        
        String::new()
    }

    fn normalize_path(path: &str) -> String {
        if path.is_empty() {
            return String::new();
        }
        
        let mut result = Vec::new();
        let segments: Vec<&str> = path.split('/').collect();
        let starts_with_slash = path.starts_with('/');
        
        for segment in segments {
            match segment {
                "" | "." => continue,
                ".." => {
                    if !result.is_empty() {
                        result.pop();
                    }
                },
                _ => result.push(segment),
            }
        }
        
        let mut normalized = result.join("/");
        if starts_with_slash {
            normalized = format!("/{}", normalized);
        }
        
        if path.ends_with('/') && !normalized.ends_with('/') && !normalized.is_empty() {
            normalized.push('/');
        }
        
        normalized
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_uri() {
        let uri = URI::parse("https://code.visualstudio.com/docs/extensions/overview#frag");
        
        assert_eq!(uri.scheme(), "https");
        assert_eq!(uri.authority(), "code.visualstudio.com");
        assert_eq!(uri.path(), "/docs/extensions/overview");
        assert_eq!(uri.query(), "");
        assert_eq!(uri.fragment(), "frag");
        
        let actual = uri.to_string(false);
        println!("Actual URI string: {}", actual);
        
        assert!(actual.contains("https://code.visualstudio.com") && 
                actual.contains("/docs/extensions/overview") && 
                actual.contains("#frag"));
    }

    #[test]
    fn test_file_uri() {
        let uri = URI::file("/users/me/c#-projects/");
        
        assert_eq!(uri.scheme(), "file");
        assert_eq!(uri.authority(), "");
        assert_eq!(uri.path(), "/users/me/c#-projects/");
        assert_eq!(uri.query(), "");
        assert_eq!(uri.fragment(), "");
        assert_eq!(
            uri.to_string(false),
            "file:///users/me/c%23-projects/"
        );
    }

    #[test]
    fn test_with_method() {
        let uri = URI::parse("https://example.com/path");
        let new_uri = uri.with(URIChange {
            scheme: Some("http".to_string()),
            path: Some("/newpath".to_string()),
            ..Default::default()
        });
        
        assert_eq!(new_uri.scheme(), "http");
        assert_eq!(new_uri.authority(), "example.com");
        assert_eq!(new_uri.path(), "/newpath");
    }

    #[test]
    fn test_utils_join_path() {
        let uri = URI::parse("https://example.com/base");
        let joined = utils::join_path(&uri, &["foo", "bar"]);
        
        assert_eq!(joined.path(), "/base/foo/bar");
    }

    #[test]
    fn test_utils_dirname() {
        let uri = URI::parse("https://example.com/path/to/file.txt");
        let dir = utils::dirname(&uri);
        
        assert_eq!(dir.path(), "/path/to");
    }

    #[test]
    fn test_utils_basename() {
        let uri = URI::parse("https://example.com/path/to/file.txt");
        let base = utils::basename(&uri);
        
        assert_eq!(base, "file.txt");
    }

    #[test]
    fn test_utils_extname() {
        let uri = URI::parse("https://example.com/path/to/file.txt");
        let ext = utils::extname(&uri);
        
        assert_eq!(ext, ".txt");
    }
}
