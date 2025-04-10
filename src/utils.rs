/*
 * Rust implementation of vscode-uri
 * https://github.com/microsoft/vscode-uri
 */

use crate::uri::{UriError, URI};

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
    pub fn join_path(uri: &URI, paths: &[&str]) -> Result<URI, UriError> {
        let mut result = uri.path().to_string();
        let mut had_trailing_slash = result.ends_with('/');

        for path in paths {
            if result.ends_with('/') {
                result.push_str(path);
            } else {
                result.push('/');
                result.push_str(path);
            }
            had_trailing_slash = path.ends_with('/');
        }

        let mut normalized = Self::normalize_path(&result);
        if had_trailing_slash && !normalized.ends_with('/') {
            normalized.push('/');
        }

        uri.with(crate::uri::URIChange {
            path: Some(normalized),
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
    pub fn resolve_path(uri: &URI, paths: &[&str]) -> Result<URI, UriError> {
        let mut base = uri.path().to_string();
        let mut slash_added = false;
        if !base.starts_with('/') {
            base = format!("/{}", base); // make the path abstract
            slash_added = true;
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

        let normalized = Self::normalize_path(&result);

        // Remove leading slash if it was added and there's no authority
        let final_path = if slash_added && uri.authority().is_empty() && normalized.starts_with('/')
        {
            if normalized == "/" {
                String::new()
            } else {
                normalized[1..].to_string()
            }
        } else {
            normalized
        };

        // Always remove trailing slash except for root
        let final_path = if final_path.len() > 1 && final_path.ends_with('/') {
            final_path[..final_path.len() - 1].to_string()
        } else {
            final_path
        };

        uri.with(crate::uri::URIChange {
            path: Some(final_path),
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
    pub fn dirname(uri: &URI) -> Result<URI, UriError> {
        let path = uri.path();
        if path.is_empty() || path == "/" {
            return Ok(uri.clone());
        }

        let mut path = path.to_string();
        // Remove trailing slashes except for root
        while path.len() > 1 && path.ends_with('/') {
            path.pop();
        }

        let new_path = match path.rfind('/') {
            Some(0) => "/",
            Some(i) => &path[..i],
            None => "",
        };

        uri.with(crate::uri::URIChange {
            path: Some(new_path.to_string()),
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
        if path.is_empty() || path == "/" {
            return String::new();
        }

        let mut path = path.to_string();
        // Remove trailing slashes except for root
        while path.len() > 1 && path.ends_with('/') {
            path.pop();
        }

        match path.rfind('/') {
            Some(last_slash) => path[last_slash + 1..].to_string(),
            None => path,
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
        let path = uri.path();
        if path.is_empty() || path == "/" {
            return String::new();
        }

        let mut path = path.to_string();
        // Remove trailing slashes except for root
        while path.len() > 1 && path.ends_with('/') {
            path.pop();
        }

        let filename = match path.rfind('/') {
            Some(last_slash) => &path[last_slash + 1..],
            None => &path,
        };

        match filename.rfind('.') {
            Some(last_dot) if last_dot > 0 && last_dot < filename.len() - 1 => {
                filename[last_dot..].to_string()
            }
            _ => String::new(),
        }
    }

    pub fn normalize_path(path: &str) -> String {
        if path.is_empty() {
            return ".".to_string();
        }

        let mut normalized = String::with_capacity(path.len());
        let had_trailing_slash = path.ends_with('/') && path != "/";
        let segments: Vec<&str> = path.split('/').collect();
        let mut stack: Vec<&str> = Vec::new();

        // Handle absolute paths
        if path.starts_with('/') {
            normalized.push('/');
        }

        for segment in segments {
            match segment {
                "" | "." => continue,
                ".." => {
                    if !stack.is_empty() && stack.last() != Some(&"..") {
                        stack.pop();
                    } else if !path.starts_with('/') {
                        // For relative paths, keep the ".." segments
                        stack.push("..");
                    }
                }
                _ => stack.push(segment),
            }
        }

        // Join the segments
        if !stack.is_empty() {
            normalized.push_str(&stack.join("/"));
        } else if !normalized.starts_with('/') {
            normalized.push('.');
        }

        // Restore trailing slash if it existed
        if had_trailing_slash {
            normalized.push('/');
        }

        normalized
    }
}
