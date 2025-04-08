/*
 * Rust implementation of vscode-uri
 * https://github.com/microsoft/vscode-uri
 */

mod uri;
mod utils;
mod char_code;
mod platform;

pub use uri::{URI, URIChange, URIComponents};
pub use utils::Utils;





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
