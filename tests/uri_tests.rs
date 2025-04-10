use vscode_uri_rs::is_windows;
use vscode_uri_rs::{URIChange, URIComponents, UriError, URI};

#[cfg(test)]
use vscode_uri_rs::platform::test_utils::set_is_windows;

type Result<T> = std::result::Result<T, UriError>;

// Helper macro to run tests in both Windows and non-Windows environments
macro_rules! test_both_platforms {
    ($name:ident, $body:expr) => {
        #[test]
        fn $name() -> Result<()> {
            // Run with is_windows = false
            set_is_windows(false);
            $body()?;

            // Run with is_windows = true
            set_is_windows(true);
            $body()?;

            Ok(())
        }
    };
}

test_both_platforms!(test_file_to_string, || {
    assert_eq!(
        URI::file("/c:/win/path")?.to_string(false).to_lowercase(),
        "file:///c%3a/win/path"
    );
    assert_eq!(
        URI::file("/C:/win/path")?.to_string(false).to_lowercase(),
        "file:///c%3a/win/path"
    );
    assert_eq!(
        URI::file("/c:/win/path/")?.to_string(false).to_lowercase(),
        "file:///c%3a/win/path/"
    );
    assert_eq!(
        URI::file("/c:/win/path")?.to_string(false).to_lowercase(),
        "file:///c%3a/win/path"
    );
    Ok(())
});

test_both_platforms!(test_uri_file_win_special, || {
    if is_windows() {
        assert_eq!(
            URI::file("c:\\win\\path")?.to_string(false),
            "file:///c%3A/win/path"
        );
        assert_eq!(
            URI::file("c:\\win/path")?.to_string(false),
            "file:///c%3A/win/path"
        );
    } else {
        assert_eq!(
            URI::file("c:\\win\\path")?.to_string(false),
            "file:///c%3A%5Cwin%5Cpath"
        );
        assert_eq!(
            URI::file("c:\\win/path")?.to_string(false),
            "file:///c%3A%5Cwin/path"
        );
    }
    Ok(())
});

test_both_platforms!(test_file_fs_path_win_special, || {
    if is_windows() {
        let uri = URI::file("c:\\win\\path")?;
        assert_eq!(uri.fs_path().to_string_lossy(), "c:\\win\\path");

        let uri = URI::file("c:\\win/path")?;
        assert_eq!(uri.fs_path().to_string_lossy(), "c:\\win\\path");

        let uri = URI::file("c:/win/path")?;
        assert_eq!(uri.fs_path().to_string_lossy(), "c:\\win\\path");

        let uri = URI::file("c:/win/path/")?;
        assert_eq!(uri.fs_path().to_string_lossy(), "c:\\win\\path\\");

        let uri = URI::file("C:/win/path")?;
        assert_eq!(uri.fs_path().to_string_lossy(), "c:\\win\\path");

        let uri = URI::file("/c:/win/path")?;
        assert_eq!(uri.fs_path().to_string_lossy(), "c:\\win\\path");

        let uri = URI::file("./c/win/path")?;
        assert_eq!(uri.fs_path().to_string_lossy(), "\\.\\c\\win\\path");
    } else {
        let uri = URI::file("c:/win/path")?;
        assert_eq!(uri.fs_path().to_string_lossy(), "c:/win/path");

        let uri = URI::file("c:/win/path/")?;
        assert_eq!(uri.fs_path().to_string_lossy(), "c:/win/path/");

        let uri = URI::file("C:/win/path")?;
        assert_eq!(uri.fs_path().to_string_lossy(), "c:/win/path");

        let uri = URI::file("/c:/win/path")?;
        assert_eq!(uri.fs_path().to_string_lossy(), "c:/win/path");

        let uri = URI::file("./c/win/path")?;
        assert_eq!(uri.fs_path().to_string_lossy(), "/./c/win/path");
    }
    Ok(())
});

test_both_platforms!(test_uri_fs_path_no_path_when_no_path, || {
    let uri = URI::parse("file://%2Fhome%2Fticino%2Fdesktop%2Fcpluscplus%2Ftest.cpp")?;

    assert_eq!(uri.authority(), "/home/ticino/desktop/cpluscplus/test.cpp");
    assert_eq!(uri.path(), "/");
    if is_windows() {
        assert_eq!(uri.fs_path().to_string_lossy(), "\\");
    } else {
        assert_eq!(uri.fs_path().to_string_lossy(), "/");
    }
    Ok(())
});

test_both_platforms!(test_http_to_string, || {
    assert_eq!(
        URI::new("http", "www.msft.com", "/my/path", "", "")?.to_string(false),
        "http://www.msft.com/my/path"
    );
    assert_eq!(
        URI::new("http", "www.msft.com", "/my/path", "", "")?.to_string(false),
        "http://www.msft.com/my/path"
    );
    assert_eq!(
        URI::new("http", "www.MSFT.com", "/my/path", "", "")?.to_string(false),
        "http://www.msft.com/my/path"
    );
    assert_eq!(
        URI::new("http", "", "my/path", "", "")?.to_string(false),
        "http:/my/path"
    );
    assert_eq!(
        URI::new("http", "", "/my/path", "", "")?.to_string(false),
        "http:/my/path"
    );
    assert_eq!(
        URI::new("http", "a-test-site.com", "/", "test=true", "")?.to_string(false),
        "http://a-test-site.com/?test%3Dtrue"
    );
    assert_eq!(
        URI::new("http", "a-test-site.com", "/", "", "test=true")?.to_string(false),
        "http://a-test-site.com/#test%3Dtrue"
    );
    Ok(())
});

test_both_platforms!(test_http_to_string_no_encode, || {
    let uri = URI::new("http", "a-test-site.com", "/", "test=true", "")?;
    let uri1 = uri.to_string(true);
    assert!(uri1.contains("http://a-test-site.com/?test=true"));

    let uri = URI::new("http", "a-test-site.com", "/", "", "test=true")?;
    let uri2 = uri.to_string(true);
    assert!(uri2.contains("http://a-test-site.com/#test=true"));

    let uri = URI::new("http", "", "/api/files/test.me", "t=1234", "")?;
    let uri3 = uri.to_string(true);
    assert!(uri3.contains("http:/api/files/test.me?t=1234"));

    let value = URI::parse("file://shares/pröjects/c%23/#l12")?;
    assert_eq!(value.authority(), "shares");
    assert_eq!(value.path(), "/pröjects/c#/");
    assert_eq!(value.fragment(), "l12");
    assert_eq!(
        value.to_string(false),
        "file://shares/pr%C3%B6jects/c%23/#l12"
    );
    assert_eq!(value.to_string(true), "file://shares/pröjects/c%23/#l12");

    let uri2 = URI::parse(value.to_string(true).as_str())?;
    let uri3 = URI::parse(value.to_string(false).as_str())?;
    assert_eq!(uri2.authority(), uri3.authority());
    assert_eq!(uri2.path(), uri3.path());
    assert_eq!(uri2.query(), uri3.query());
    assert_eq!(uri2.fragment(), uri3.fragment());
    Ok(())
});

test_both_platforms!(test_with_identity, || {
    let uri = URI::parse("foo:bar/path")?;

    let uri2 = uri.with(URIChange::default())?;
    assert_eq!(uri, uri2);
    let uri2 = uri.with(URIChange {
        scheme: Some("foo".to_string()),
        path: Some("bar/path".to_string()),
        ..Default::default()
    })?;
    assert_eq!(uri, uri2);
    let uri2 = uri.with(URIChange {
        scheme: Some("foo".to_string()),
        path: Some("bar/path".to_string()),
        ..Default::default()
    })?;
    assert_eq!(uri, uri2);
    let uri2 = uri.with(URIChange {
        scheme: Some("foo".to_string()),
        path: Some("bar/path".to_string()),
        ..Default::default()
    })?;
    assert_eq!(uri, uri2);
    Ok(())
});

test_both_platforms!(test_with_changes, || {
    let uri = URI::parse("before:some/file/path")?;
    let uri2 = uri.with(URIChange {
        scheme: Some("after".to_string()),
        ..Default::default()
    })?;
    assert_eq!(uri2.to_string(false), "after:some/file/path");

    let uri = URI::from(&URIComponents {
        scheme: "s".to_string(),
        authority: "".to_string(),
        path: "/api/files/test.me".to_string(),
        query: "t=1234".to_string(),
        fragment: "".to_string(),
    })?;
    let uri2 = uri.with(URIChange {
        scheme: Some("http".to_string()),
        ..Default::default()
    })?;
    assert_eq!(uri2.to_string(false), "http:/api/files/test.me?t%3D1234");

    let uri = URI::from(&URIComponents {
        scheme: "s".to_string(),
        authority: "".to_string(),
        path: "/api/files/test.me".to_string(),
        query: "t=1234".to_string(),
        fragment: "".to_string(),
    })?;
    let uri2 = uri.with(URIChange {
        scheme: Some("http".to_string()),
        ..Default::default()
    })?;
    assert_eq!(uri2.to_string(false), "http:/api/files/test.me?t%3D1234");

    let uri = URI::from(&URIComponents {
        scheme: "s".to_string(),
        authority: "".to_string(),
        path: "/api/files/test.me".to_string(),
        query: "t=1234".to_string(),
        fragment: "".to_string(),
    })?;
    let uri2 = uri.with(URIChange {
        scheme: Some("https".to_string()),
        ..Default::default()
    })?;
    assert_eq!(uri2.to_string(false), "https:/api/files/test.me?t%3D1234");

    let uri = URI::from(&URIComponents {
        scheme: "s".to_string(),
        authority: "".to_string(),
        path: "/api/files/test.me".to_string(),
        query: "t=1234".to_string(),
        fragment: "".to_string(),
    })?;
    let uri2 = uri.with(URIChange {
        scheme: Some("HTTP".to_string()),
        ..Default::default()
    })?;
    assert_eq!(uri2.to_string(false), "HTTP:/api/files/test.me?t%3D1234");

    let uri = URI::from(&URIComponents {
        scheme: "s".to_string(),
        authority: "".to_string(),
        path: "/api/files/test.me".to_string(),
        query: "t=1234".to_string(),
        fragment: "".to_string(),
    })?;
    let uri2 = uri.with(URIChange {
        scheme: Some("HTTPS".to_string()),
        ..Default::default()
    })?;
    assert_eq!(uri2.to_string(false), "HTTPS:/api/files/test.me?t%3D1234");

    let uri = URI::from(&URIComponents {
        scheme: "s".to_string(),
        authority: "".to_string(),
        path: "/api/files/test.me".to_string(),
        query: "t=1234".to_string(),
        fragment: "".to_string(),
    })?;
    let uri2 = uri.with(URIChange {
        scheme: Some("boo".to_string()),
        ..Default::default()
    })?;
    assert_eq!(uri2.to_string(false), "boo:/api/files/test.me?t%3D1234");

    Ok(())
});

test_both_platforms!(test_with_remove_components, || {
    let uri = URI::parse("scheme://authority/path")?;
    let uri1 = uri
        .with(URIChange {
            authority: Some("".to_string()),
            ..Default::default()
        })?
        .to_string(false);
    assert!(uri1 == "scheme:/path" || uri1 == "scheme:///path");

    let uri = URI::parse("scheme:/path")?;
    let uri2 = uri
        .with(URIChange {
            authority: Some("authority".to_string()),
            ..Default::default()
        })?
        .with(URIChange {
            authority: Some("".to_string()),
            ..Default::default()
        })?
        .to_string(false);
    assert!(uri2 == "scheme:/path" || uri2 == "scheme:///path");

    let uri = URI::parse("scheme:/path")?;
    let uri3 = uri
        .with(URIChange {
            authority: Some("authority".to_string()),
            ..Default::default()
        })?
        .with(URIChange {
            path: Some("".to_string()),
            ..Default::default()
        })?
        .to_string(false);
    assert!(uri3 == "scheme://authority" || uri3 == "scheme://authority/");

    let uri = URI::parse("scheme:/path")?;
    let uri4 = uri
        .with(URIChange {
            authority: Some("authority".to_string()),
            ..Default::default()
        })?
        .with(URIChange {
            path: Some("".to_string()),
            ..Default::default()
        })?
        .to_string(false);
    assert!(uri4 == "scheme://authority" || uri4 == "scheme://authority/");

    let uri = URI::parse("scheme:/path")?;
    let uri5 = uri
        .with(URIChange {
            authority: Some("authority".to_string()),
            ..Default::default()
        })?
        .with(URIChange {
            authority: Some("".to_string()),
            ..Default::default()
        })?
        .to_string(false);
    assert!(uri5 == "scheme:/path" || uri5 == "scheme:///path");

    let uri = URI::parse("scheme:/path")?;
    let uri6 = uri
        .with(URIChange {
            authority: Some("authority".to_string()),
            ..Default::default()
        })?
        .with(URIChange {
            authority: Some("".to_string()),
            ..Default::default()
        })?
        .to_string(false);
    assert!(uri6 == "scheme:/path" || uri6 == "scheme:///path");

    Ok(())
});

#[test]
fn test_with_validation_scheme() {
    let uri = URI::parse("foo:bar/path").unwrap();
    assert!(uri
        .with(URIChange {
            scheme: Some("fai:l".to_string()),
            ..Default::default()
        })
        .is_err());
    assert!(uri
        .with(URIChange {
            scheme: Some("fäil".to_string()),
            ..Default::default()
        })
        .is_err());
    assert!(uri
        .with(URIChange {
            authority: Some("fail".to_string()),
            ..Default::default()
        })
        .is_err());
    assert!(uri
        .with(URIChange {
            path: Some("//fail".to_string()),
            ..Default::default()
        })
        .is_err());
}

test_both_platforms!(test_parse, || {
    let value = URI::parse("http:/api/files/test.me?t=1234")?;
    assert_eq!(value.scheme(), "http");
    assert_eq!(value.authority(), "");
    assert_eq!(value.path(), "/api/files/test.me");
    assert_eq!(value.query(), "t=1234");
    assert_eq!(value.fragment(), "");

    let value = URI::parse("http://api/files/test.me?t=1234")?;
    assert_eq!(value.scheme(), "http");
    assert_eq!(value.authority(), "api");
    assert_eq!(value.path(), "/files/test.me");
    assert_eq!(value.query(), "t=1234");
    assert_eq!(value.fragment(), "");

    let value = URI::parse("file:///c:/test/me")?;
    assert_eq!(value.scheme(), "file");
    assert_eq!(value.authority(), "");
    assert_eq!(value.path(), "/c:/test/me");
    assert_eq!(value.fragment(), "");
    assert_eq!(value.query(), "");
    if is_windows() {
        assert_eq!(value.fs_path().to_string_lossy(), "c:\\test\\me");
    } else {
        assert_eq!(value.fs_path().to_string_lossy(), "c:/test/me");
    }

    let value = URI::parse("file://shares/files/c%23/p.cs")?;
    assert_eq!(value.scheme(), "file");
    assert_eq!(value.authority(), "shares");
    assert_eq!(value.path(), "/files/c#/p.cs");
    assert_eq!(value.fragment(), "");
    assert_eq!(value.query(), "");
    if is_windows() {
        assert_eq!(
            value.fs_path().to_string_lossy(),
            "\\\\shares\\files\\c#\\p.cs"
        );
    } else {
        assert_eq!(value.fs_path().to_string_lossy(), "//shares/files/c#/p.cs");
    }

    let value = URI::parse(
        "file:///c:/Source/Z%C3%BCrich%20or%20Zurich%20(%CB%88zj%CA%8A%C9%99r%C9%AAk,/Code/resources/app/plugins/c%23/plugin.json",
    )?;
    assert_eq!(value.scheme(), "file");
    assert_eq!(value.authority(), "");
    assert_eq!(
        value.path(),
        "/c:/Source/Zürich or Zurich (ˈzjʊərɪk,/Code/resources/app/plugins/c#/plugin.json"
    );
    assert_eq!(value.fragment(), "");
    assert_eq!(value.query(), "");

    let value = URI::parse("file:///c:/test %25/path")?;
    assert_eq!(value.scheme(), "file");
    assert_eq!(value.authority(), "");
    assert_eq!(value.path(), "/c:/test %/path");
    assert_eq!(value.fragment(), "");
    assert_eq!(value.query(), "");

    let value = URI::parse("inmemory:")?;
    assert_eq!(value.scheme(), "inmemory");
    assert_eq!(value.authority(), "");
    assert_eq!(value.path(), "");
    assert_eq!(value.query(), "");
    assert_eq!(value.fragment(), "");

    let value = URI::parse("foo:api/files/test")?;
    assert_eq!(value.scheme(), "foo");
    assert_eq!(value.authority(), "");
    assert_eq!(value.path(), "api/files/test");
    assert_eq!(value.query(), "");
    assert_eq!(value.fragment(), "");

    let value = URI::parse("file:?q")?;
    assert_eq!(value.scheme(), "file");
    assert_eq!(value.authority(), "");
    assert_eq!(value.path(), "/");
    assert_eq!(value.query(), "q");
    assert_eq!(value.fragment(), "");

    let value = URI::parse("file:#d")?;
    assert_eq!(value.scheme(), "file");
    assert_eq!(value.authority(), "");
    assert_eq!(value.path(), "/");
    assert_eq!(value.query(), "");
    assert_eq!(value.fragment(), "d");

    let value = URI::parse("f3ile:#d")?;
    assert_eq!(value.scheme(), "f3ile");
    assert_eq!(value.authority(), "");
    assert_eq!(value.path(), "");
    assert_eq!(value.query(), "");
    assert_eq!(value.fragment(), "d");

    let value = URI::parse("foo+bar:path")?;
    assert_eq!(value.scheme(), "foo+bar");
    assert_eq!(value.authority(), "");
    assert_eq!(value.path(), "path");
    assert_eq!(value.query(), "");
    assert_eq!(value.fragment(), "");

    let value = URI::parse("foo-bar:path")?;
    assert_eq!(value.scheme(), "foo-bar");
    assert_eq!(value.authority(), "");
    assert_eq!(value.path(), "path");
    assert_eq!(value.query(), "");
    assert_eq!(value.fragment(), "");

    let value = URI::parse("foo.bar:path")?;
    assert_eq!(value.scheme(), "foo.bar");
    assert_eq!(value.authority(), "");
    assert_eq!(value.path(), "path");
    assert_eq!(value.query(), "");
    assert_eq!(value.fragment(), "");
    Ok(())
});

test_both_platforms!(test_parse_disallow_path_when_no_authority, || {
    assert!(URI::parse("file:////shares/files/p.cs").is_err());
    Ok(())
});

test_both_platforms!(test_file_win_special, || {
    if is_windows() {
        let value = URI::file("c:\\test\\drive")?;
        assert_eq!(value.path(), "/c:/test/drive");
        assert_eq!(value.to_string(false), "file:///c%3A/test/drive");

        let value = URI::file("\\\\shäres\\path\\c#\\plugin.json")?;
        assert_eq!(value.scheme(), "file");
        assert_eq!(value.authority(), "shäres");
        assert_eq!(value.path(), "/path/c#/plugin.json");
        assert_eq!(value.fragment(), "");
        assert_eq!(value.query(), "");
        assert_eq!(
            value.to_string(false),
            "file://sh%C3%A4res/path/c%23/plugin.json"
        );

        let value = URI::file("\\\\localhost\\c$\\GitDevelopment\\express")?;
        assert_eq!(value.scheme(), "file");
        assert_eq!(value.path(), "/c$/GitDevelopment/express");
        assert_eq!(
            value.fs_path().to_string_lossy(),
            "\\\\localhost\\c$\\GitDevelopment\\express"
        );
        assert_eq!(value.query(), "");
        assert_eq!(value.fragment(), "");
        assert_eq!(
            value.to_string(false),
            "file://localhost/c%24/GitDevelopment/express"
        );

        let value = URI::file("c:\\test with %\\path")?;
        assert_eq!(value.path(), "/c:/test with %/path");
        assert_eq!(
            value.to_string(false),
            "file:///c%3A/test%20with%20%25/path"
        );

        let value = URI::file("c:\\test with %25\\path")?;
        assert_eq!(value.path(), "/c:/test with %25/path");
        assert_eq!(
            value.to_string(false),
            "file:///c%3A/test%20with%20%2525/path"
        );

        let value = URI::file("c:\\test with %25\\c#code")?;
        assert_eq!(value.path(), "/c:/test with %25/c#code");
        assert_eq!(
            value.to_string(false),
            "file:///c%3A/test%20with%20%2525/c%23code"
        );

        let value = URI::file("\\\\shares")?;
        assert_eq!(value.scheme(), "file");
        assert_eq!(value.authority(), "shares");
        assert_eq!(value.path(), "/");
        assert_eq!(value.to_string(false), "file://shares/");

        let value = URI::file("\\\\shares\\")?;
        assert_eq!(value.scheme(), "file");
        assert_eq!(value.authority(), "shares");
        assert_eq!(value.path(), "/");
        assert_eq!(value.to_string(false), "file://shares/");
    }
    Ok(())
});

test_both_platforms!(test_file_always_slash, || {
    let value = URI::file("a.file")?;
    assert_eq!(value.scheme(), "file");
    assert_eq!(value.authority(), "");
    assert_eq!(value.path(), "/a.file");
    assert_eq!(value.to_string(false), "file:///a.file");

    let value = URI::parse(&value.to_string(false))?;
    assert_eq!(value.scheme(), "file");
    assert_eq!(value.authority(), "");
    assert_eq!(value.path(), "/a.file");
    assert_eq!(value.to_string(false), "file:///a.file");
    Ok(())
});

test_both_platforms!(test_uri_to_string_user_information_in_authority, || {
    let value = URI::parse("http://foo:bar@localhost/far")?;
    assert_eq!(value.to_string(false), "http://foo:bar@localhost/far");

    let value = URI::parse("http://foo@localhost/far")?;
    assert_eq!(value.to_string(false), "http://foo@localhost/far");

    let value = URI::parse("http://foo:bAr@localhost:8080/far")?;
    assert_eq!(value.to_string(false), "http://foo:bAr@localhost:8080/far");

    let value = URI::parse("http://foo@localhost:8080/far")?;
    assert_eq!(value.to_string(false), "http://foo@localhost:8080/far");

    let value = URI::from(&URIComponents {
        scheme: "http".to_string(),
        authority: "föö:bör@löcalhost:8080".to_string(),
        path: "/far".to_string(),
        query: "".to_string(),
        fragment: "".to_string(),
    })?;
    assert_eq!(
        value.to_string(false),
        "http://f%C3%B6%C3%B6:b%C3%B6r@l%C3%B6calhost:8080/far"
    );
    Ok(())
});

test_both_platforms!(test_correct_file_uri_to_file_path2, || {
    let test = |input: &str, expected: &str| {
        let value = URI::parse(input).unwrap();
        assert_eq!(value.fs_path().to_string_lossy(), expected);
        let value2 = URI::file(value.fs_path()).unwrap();
        assert_eq!(value2.fs_path().to_string_lossy(), expected);
        assert_eq!(value.to_string(false), value2.to_string(false));
    };

    if is_windows() {
        test("file:///c:/alex.txt", "c:\\alex.txt");
        test("file:///c:/Source/Z%C3%BCrich%20or%20Zurich%20(%CB%88zj%CA%8A%C9%99r%C9%AAk,/Code/resources/app/plugins", "c:\\Source\\Zürich or Zurich (ˈzjʊərɪk,\\Code\\resources\\app\\plugins");
        test(
            "file://monacotools/folder/isi.txt",
            "\\\\monacotools\\folder\\isi.txt",
        );
        test(
            "file://monacotools1/certificates/SSL/",
            "\\\\monacotools1\\certificates\\SSL\\",
        );
    } else {
        test("file:///c:/alex.txt", "c:/alex.txt");
        test("file:///c:/Source/Z%C3%BCrich%20or%20Zurich%20(%CB%88zj%CA%8A%C9%99r%C9%AAk,/Code/resources/app/plugins", "c:/Source/Zürich or Zurich (ˈzjʊərɪk,/Code/resources/app/plugins");
        test(
            "file://monacotools/folder/isi.txt",
            "//monacotools/folder/isi.txt",
        );
        test(
            "file://monacotools1/certificates/SSL/",
            "//monacotools1/certificates/SSL/",
        );
    }
    Ok(())
});

test_both_platforms!(test_uri_to_string_only_scheme_and_query, || {
    let value = URI::parse("stuff:?qüery")?;
    assert_eq!(value.to_string(false), "stuff:?q%C3%BCery");
    Ok(())
});

test_both_platforms!(test_uri_to_string_upper_case_percent_espaces, || {
    let value = URI::parse("file://sh%c3%a4res/path")?;
    assert_eq!(value.to_string(false), "file://sh%C3%A4res/path");
    Ok(())
});

test_both_platforms!(test_uri_to_string_lower_case_windows_drive_letter, || {
    let value = URI::parse("untitled:c:/Users/jrieken/Code/abc.txt")?;
    assert_eq!(
        value.to_string(false),
        "untitled:c%3A/Users/jrieken/Code/abc.txt"
    );
    Ok(())
});

test_both_platforms!(test_uri_to_string_escape_all_the_bits, || {
    let value = URI::file("/Users/jrieken/Code/_samples/18500/Mödel + Other Thîngß/model.js")?;
    assert_eq!(value.to_string(false), "file:///Users/jrieken/Code/_samples/18500/M%C3%B6del%20%2B%20Other%20Th%C3%AEng%C3%9F/model.js");
    Ok(())
});

test_both_platforms!(test_uri_to_string_dont_encode_port, || {
    let value = URI::parse("http://localhost:8080/far")?;
    assert_eq!(value.to_string(false), "http://localhost:8080/far");

    let value = URI::from(&URIComponents {
        scheme: "http".to_string(),
        authority: "löcalhost:8080".to_string(),
        path: "/far".to_string(),
        query: "".to_string(),
        fragment: "".to_string(),
    })?;
    assert_eq!(value.to_string(false), "http://l%C3%B6calhost:8080/far");
    Ok(())
});

test_both_platforms!(test_uri_http_query_and_to_string, || {
    let uri = URI::parse("https://go.microsoft.com/fwlink/?LinkId=518008")?;
    assert_eq!(uri.query(), "LinkId=518008");
    assert_eq!(
        uri.to_string(true),
        "https://go.microsoft.com/fwlink/?LinkId=518008"
    );
    assert_eq!(
        uri.to_string(false),
        "https://go.microsoft.com/fwlink/?LinkId%3D518008"
    );

    let uri2 = URI::parse(&uri.to_string(false))?;
    assert_eq!(uri2.query(), "LinkId=518008");
    assert_eq!(uri2.query(), uri.query());

    let uri = URI::parse("https://go.microsoft.com/fwlink/?LinkId=518008&foö&ké¥=üü")?;
    assert_eq!(uri.query(), "LinkId=518008&foö&ké¥=üü");
    assert_eq!(
        uri.to_string(true),
        "https://go.microsoft.com/fwlink/?LinkId=518008&foö&ké¥=üü"
    );
    assert_eq!(uri.to_string(false), "https://go.microsoft.com/fwlink/?LinkId%3D518008%26fo%C3%B6%26k%C3%A9%C2%A5%3D%C3%BC%C3%BC");

    let uri2 = URI::parse(&uri.to_string(false))?;
    assert_eq!(uri2.query(), "LinkId=518008&foö&ké¥=üü");
    assert_eq!(uri2.query(), uri.query());

    let uri = URI::parse("https://twitter.com/search?src=typd&q=%23tag")?;
    assert_eq!(
        uri.to_string(true),
        "https://twitter.com/search?src=typd&q=%23tag"
    );
    Ok(())
});

test_both_platforms!(test_class_uri_cannot_represent_relative_file_paths, || {
    let path = "/foo/bar";
    assert_eq!(URI::file(path)?.path(), path);
    let path = "foo/bar";
    assert_eq!(URI::file(path)?.path(), "/foo/bar");
    let path = "./foo/bar";
    assert_eq!(URI::file(path)?.path(), "/./foo/bar"); // missing normalization

    let file_uri1 = URI::parse("file:foo/bar")?;
    assert_eq!(file_uri1.path(), "/foo/bar");
    assert_eq!(file_uri1.authority(), "");
    let uri = file_uri1.to_string(false);
    assert_eq!(uri, "file:///foo/bar");
    let file_uri2 = URI::parse(&uri)?;
    assert_eq!(file_uri2.path(), "/foo/bar");
    assert_eq!(file_uri2.authority(), "");
    Ok(())
});

test_both_platforms!(
    test_ctrl_click_to_follow_hash_query_param_url_gets_urlencoded,
    || {
        let input = "http://localhost:3000/#/foo?bar=baz";
        let uri = URI::parse(input)?;
        assert_eq!(uri.to_string(true), input);

        let input = "http://localhost:3000/foo?bar=baz";
        let uri = URI::parse(input)?;
        assert_eq!(uri.to_string(true), input);

        Ok(())
    }
);

test_both_platforms!(test_unable_to_open_a0_txt_uri_malformed, || {
    let uri = URI::file("/foo/%A0.txt")?;
    let uri2 = URI::parse(&uri.to_string(false))?;
    assert_eq!(uri.scheme(), uri2.scheme());
    assert_eq!(uri.path(), uri2.path());

    let uri = URI::file("/foo/%2e.txt")?;
    let uri2 = URI::parse(&uri.to_string(false))?;
    assert_eq!(uri.scheme(), uri2.scheme());
    assert_eq!(uri.path(), uri2.path());

    let uri = URI::parse("file://some/%.txt")?;
    assert_eq!(uri.to_string(false), "file://some/%25.txt");

    let uri = URI::parse("file://some/%A0.txt")?;
    assert_eq!(uri.to_string(false), "file://some/%25A0.txt");

    Ok(())
});
