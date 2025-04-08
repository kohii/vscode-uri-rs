/*
 * Rust implementation of vscode-uri
 * https://github.com/microsoft/vscode-uri
 */

use crate::uri::URI;

pub struct Utils;

impl Utils {
    /**
     * Joins one or more input paths to the path of URI. 
     * '/' is used as the directory separation character. 
     * 
     * The resolved path will be normalized. That means:
     *  - all '..' and '.' segments are resolved.
     *  - multiple, sequential occurences of '/' are replaced by a single instance of '/'.
     *  - trailing separators are preserved.
     * 
     * @param uri The input URI.
     * @param paths The paths to be joined with the path of URI.
     * @returns A URI with the joined path. All other properties of the URI (scheme, authority, query, fragments, ...) will be taken from the input URI.
     */
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
        
        result = Self::normalize_path(&result);
        
        uri.with(crate::uri::URIChange {
            path: Some(result),
            ..Default::default()
        })
    }

    /**
     * Resolves one or more paths against the path of a URI. 
     * '/' is used as the directory separation character. 
     * 
     * The resolved path will be normalized. That means:
     *  - all '..' and '.' segments are resolved. 
     *  - multiple, sequential occurences of '/' are replaced by a single instance of '/'.
     *  - trailing separators are removed.
     * 
     * @param uri The input URI.
     * @param paths The paths to resolve against the path of URI.
     * @returns A URI with the resolved path. All other properties of the URI (scheme, authority, query, fragments, ...) will be taken from the input URI.
     */
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
        
        uri.with(crate::uri::URIChange {
            path: Some(result),
            ..Default::default()
        })
    }

    /**
     * Returns a URI where the path is the directory name of the input uri, similar to the Unix dirname command. 
     * In the path, '/' is recognized as the directory separation character. Trailing directory separators are ignored.
     * The orignal URI is returned if the URIs path is empty or does not contain any path segments.
     * 
     * @param uri The input URI.
     * @return The last segment of the URIs path.
     */
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
        
        uri.with(crate::uri::URIChange {
            path: Some(result),
            ..Default::default()
        })
    }

    /**
     * Returns the last segment of the path of a URI, similar to the Unix basename command. 
     * In the path, '/' is recognized as the directory separation character. Trailing directory separators are ignored.
     * The empty string is returned if the URIs path is empty or does not contain any path segments.
     * 
     * @param uri The input URI.
     * @return The base name of the URIs path.
     */
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

    /**
     * Returns the extension name of the path of a URI, similar to the Unix extname command. 
     * In the path, '/' is recognized as the directory separation character. Trailing directory separators are ignored.
     * The empty string is returned if the URIs path is empty or does not contain any path segments.
     * 
     * @param uri The input URI.
     * @return The extension name of the URIs path.
     */
    pub fn extname(uri: &URI) -> String {
        let base = Self::basename(uri);
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
