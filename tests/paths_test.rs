use vscode_uri_rs::{URIComponents, UriError, Utils, URI};
type Result<T> = std::result::Result<T, UriError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_join() -> Result<()> {
        fn assert_join(uri: &str, paths: &[&str], expected: &str) -> Result<()> {
            let test_uri = URI::parse(uri)?;
            let mut joined = test_uri;
            for path in paths {
                joined = Utils::join_path(&joined, &[path])?;
            }
            assert_eq!(joined.to_string(false), expected);
            Ok(())
        }

        assert_join("foo://a/foo/bar", &["x"], "foo://a/foo/bar/x")?;
        assert_join("foo://a/foo/bar/", &["x"], "foo://a/foo/bar/x")?;
        assert_join("foo://a/foo/bar/", &["/x"], "foo://a/foo/bar/x")?;
        assert_join("foo://a/foo/bar/", &["x/"], "foo://a/foo/bar/x/")?;
        assert_join("foo://a/foo/bar/", &["x", "y"], "foo://a/foo/bar/x/y")?;
        assert_join("foo://a/foo/bar/", &["x/", "/y"], "foo://a/foo/bar/x/y")?;
        assert_join("foo://a/foo/bar/", &[".", "/y"], "foo://a/foo/bar/y")?;
        assert_join("foo://a/foo/bar/", &["x/y/z", ".."], "foo://a/foo/bar/x/y")?;
        assert_join(
            "untitled:untitled-1",
            &["..", "untitled-2"],
            "untitled:untitled-2",
        )?;

        Ok(())
    }

    #[test]
    fn test_resolve() -> Result<()> {
        fn assert_resolve(uri: &str, path: &str, expected: &str) -> Result<()> {
            let test_uri = URI::parse(uri)?;
            let resolved = Utils::resolve_path(&test_uri, &[path])?;
            assert_eq!(resolved.to_string(false), expected);
            Ok(())
        }

        assert_resolve("foo://a/foo/bar", "x", "foo://a/foo/bar/x")?;
        assert_resolve("foo://a/foo/bar/", "x", "foo://a/foo/bar/x")?;
        assert_resolve("foo://a/foo/bar/", "/x", "foo://a/x")?;
        assert_resolve("foo://a/foo/bar/", "x/", "foo://a/foo/bar/x")?;

        assert_resolve("foo://a", "x/", "foo://a/x")?;
        assert_resolve("foo://a", "/x/", "foo://a/x")?;

        assert_resolve("foo://a/b", "/x/..//y/.", "foo://a/y")?;
        assert_resolve("foo://a/b", "x/..//y/.", "foo://a/b/y")?;
        assert_resolve("untitled:untitled-1", "../foo", "untitled:foo")?;
        assert_resolve("untitled:", "foo", "untitled:foo")?;
        assert_resolve("untitled:", "..", "untitled:")?;
        assert_resolve("untitled:", "/foo", "untitled:foo")?;
        assert_resolve("untitled:/", "/foo", "untitled:/foo")?;

        Ok(())
    }

    #[test]
    fn test_normalize() -> Result<()> {
        fn assert_normalize(path: &str, expected: &str) -> Result<()> {
            let components = URIComponents {
                scheme: "foo".to_string(),
                path: path.to_string(),
                authority: if path.starts_with('/') {
                    "bar".to_string()
                } else {
                    String::new()
                },
                query: String::new(),
                fragment: String::new(),
            };
            let test_uri = URI::from(&components)?;
            let normalized = Utils::join_path(&test_uri, &[])?;
            assert_eq!(normalized.path(), expected);
            Ok(())
        }

        assert_normalize("a", "a")?;
        assert_normalize("/a", "/a")?;
        assert_normalize("a/", "a/")?;
        assert_normalize("a/b", "a/b")?;
        assert_normalize("/a/foo/bar/x", "/a/foo/bar/x")?;
        assert_normalize("/a/foo/bar//x", "/a/foo/bar/x")?;
        assert_normalize("/a/foo/bar///x", "/a/foo/bar/x")?;
        assert_normalize("/a/foo/bar/x/", "/a/foo/bar/x/")?;
        assert_normalize("a/foo/bar/x/", "a/foo/bar/x/")?;
        assert_normalize("a/foo/bar/x//", "a/foo/bar/x/")?;
        assert_normalize("//a/foo/bar/x//", "/a/foo/bar/x/")?;
        assert_normalize("a/.", "a")?;
        assert_normalize("a/..", ".")?;
        assert_normalize("a/./b", "a/b")?;
        assert_normalize("a/././b", "a/b")?;
        assert_normalize("a/n/../b", "a/b")?;
        assert_normalize("a/n/../", "a/")?;
        assert_normalize("/a/n/../..", "/")?;
        assert_normalize("/a/n/../../..", "/")?;
        assert_normalize("..", "..")?;
        assert_normalize("/..", "/")?;
        assert_normalize("untitled-1/foo/bar/.", "untitled-1/foo/bar")?;

        Ok(())
    }

    #[test]
    fn test_extname() -> Result<()> {
        fn assert_extname(input: &str, expected: &str) -> Result<()> {
            let test_uri = URI::parse(input)?;
            let ext = Utils::extname(&test_uri);
            assert_eq!(ext, expected);
            Ok(())
        }

        assert_extname("foo://a/foo/bar", "")?;
        assert_extname("foo://a/foo/bar.foo", ".foo")?;
        assert_extname("foo://a/foo/.foo", "")?;
        assert_extname("foo://a/foo/a.foo/", ".foo")?;
        assert_extname("foo://a/foo/a.foo//", ".foo")?;
        assert_extname("untitled:untitled-1", "")?;

        Ok(())
    }

    #[test]
    fn test_basename() -> Result<()> {
        fn assert_basename(input: &str, expected: &str) -> Result<()> {
            let test_uri = URI::parse(input)?;
            let base = Utils::basename(&test_uri);
            assert_eq!(base, expected);
            Ok(())
        }

        assert_basename("foo://a/some/file/test.txt", "test.txt")?;
        assert_basename("foo://a/some/file/", "file")?;
        assert_basename("foo://a/some/file///", "file")?;
        assert_basename("foo://a/some/file", "file")?;
        assert_basename("foo://a/some", "some")?;
        assert_basename("foo://a/", "")?;
        assert_basename("foo://a", "")?;
        assert_basename("untitled:untitled-1", "untitled-1")?;

        Ok(())
    }

    #[test]
    fn test_dirname() -> Result<()> {
        fn assert_dirname(input: &str, expected: &str) -> Result<()> {
            let test_uri = URI::parse(input)?;
            let dir = Utils::dirname(&test_uri)?;
            assert_eq!(dir.to_string(false), expected);
            Ok(())
        }

        assert_dirname("foo://a/some/file/test.txt", "foo://a/some/file")?;
        assert_dirname("foo://a/some/file/", "foo://a/some")?;
        assert_dirname("foo://a/some/file///", "foo://a/some")?;
        assert_dirname("foo://a/some/file", "foo://a/some")?;
        assert_dirname("foo://a/some", "foo://a/")?;
        assert_dirname("foo://a/", "foo://a/")?;
        assert_dirname("foo://a", "foo://a")?;
        assert_dirname("foo://", "foo:")?;
        assert_dirname("untitled:untitled-1", "untitled:")?;

        Ok(())
    }
}
