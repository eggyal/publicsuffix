extern crate rspec;

use {List, request};
use self::rspec::context::rdescribe;

macro_rules! pass {
    () => { Ok(()) as Result<(), ()> }
}

#[test]
fn list_behaviour() {
    let list = List::fetch().unwrap();

    rdescribe("the list", |ctx| {
        ctx.it("should not be empty", || {
            assert!(!list.all().is_empty());
            pass!()
        });

        ctx.it("should have ICANN domains", || {
            assert!(!list.icann().is_empty());
            pass!()
        });

        ctx.it("should have private domains", || {
            assert!(!list.private().is_empty());
            pass!()
        });

        ctx.it("should have at least 1000 domains", || {
            assert!(list.all().len() > 1000);
            pass!()
        });
    });

    rdescribe("the official test", |_| {
        let tests = "https://raw.githubusercontent.com/publicsuffix/list/master/tests/tests.txt";
        let body = request(tests).unwrap();

        let mut parse = false;

        for (i, line) in body.lines().enumerate() {
            match line {
                line if line.trim().is_empty() => { parse = true; continue; }
                line if line.starts_with("//") => { continue; }
                line => {
                    if !parse { continue; }
                    let mut test = line.split_whitespace().peekable();
                    if test.peek().is_none() {
                        continue;
                    }
                    let input = match test.next() {
                        Some("null") => "",
                        Some(res) => res,
                        None => { panic!(format!("line {} of the test file doesn't seem to be valid", i)); },
                    };
                    let (expected_root, expected_suffix) = match test.next() {
                        Some("null") => (None, None),
                        Some(root) => {
                            let suffix = {
                                let parts: Vec<&str> = root.split('.').rev().collect();
                                (&parts[..parts.len()-1]).iter().rev()
                                    .map(|part| *part)
                                    .collect::<Vec<_>>()
                                    .join(".")
                            };
                            (Some(root.to_string()), Some(suffix.to_string()))
                        },
                        None => { panic!(format!("line {} of the test file doesn't seem to be valid", i)); },
                    };
                    let (found_root, found_suffix) = match list.parse_domain(input) {
                        Ok(domain) => {
                            let found_root = match domain.root() {
                                Some(found) => Some(found.to_string()),
                                None => None,
                            };
                            let found_suffix = match domain.suffix() {
                                Some(found) => Some(found.to_string()),
                                None => None,
                            };
                            (found_root, found_suffix)
                        },
                        Err(_) => (None, None),
                    };
                    if expected_root != found_root || (expected_root.is_some() && expected_suffix != found_suffix) {
                        let msg = format!("\n\nGiven `{}`:\nWe expected root domain to be `{:?}` and suffix be `{:?}`\nBut instead, we have `{:?}` as root domain and `{:?}` as suffix.\nWe are on line {} of `test_psl.txt`.\n\n",
                                          input, expected_root, expected_suffix, found_root, found_suffix, i+1);
                        panic!(msg);
                    }
                }
            }
        }
    });

    rdescribe("the domain", |ctx| {
        ctx.it("should allow fully qualified domain names", || {
            assert!(list.parse_domain("example.com.").is_ok());
            pass!()
        });

        ctx.it("should not allow more than 1 trailing dots", || {
            assert!(list.parse_domain("example.com..").is_err());
            pass!()
        });

        ctx.it("should not contain spaces", || {
            assert!(list.parse_domain("exa mple.com").is_err());
            pass!()
        });

        ctx.it("should not start with a dash", || {
            assert!(list.parse_domain("-example.com").is_err());
            pass!()
        });

        ctx.it("should not contain /", || {
            assert!(list.parse_domain("exa/mple.com").is_err());
            pass!()
        });

        ctx.it("should not have a label > 63 characters", || {
            let mut too_long_domain = String::from("a");
            for _ in 0..64 {
                too_long_domain.push_str("a");
            }
            too_long_domain.push_str(".com");
            assert!(list.parse_domain(&too_long_domain).is_err());
            pass!()
        });

        ctx.it("should not be an IPv4 address", || {
            assert!(list.parse_domain("127.38.53.247").is_err());
            pass!()
        });

        ctx.it("should not be an IPv6 address", || {
            assert!(list.parse_domain("fd79:cdcb:38cc:9dd:f686:e06d:32f3:c123").is_err());
            pass!()
        });

        ctx.it("should allow numbers only labels that are not the tld", || {
            assert!(list.parse_domain("127.com").is_ok());
            pass!()
        });

        ctx.it("should not have more than 127 labels", || {
            let mut too_many_labels_domain = String::from("a");
            for _ in 0..126 {
                too_many_labels_domain.push_str(".a");
            }
            too_many_labels_domain.push_str(".com");
            assert!(list.parse_domain(&too_many_labels_domain).is_err());
            pass!()
        });

        ctx.it("should not have more than 253 characters", || {
            let mut too_many_chars_domain = String::from("aaaaa");
            for _ in 0..50 {
                too_many_chars_domain.push_str(".aaaaaa");
            }
            too_many_chars_domain.push_str(".com");
            assert!(list.parse_domain(&too_many_chars_domain).is_err());
            pass!()
        });
    });
}
