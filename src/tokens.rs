use regex::Regex;

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    LeftBracket,
    RightBracket,
    I64(i64),
    F64(f64),
    Term(std::string::String),
    DeferredTerm(std::string::String),
    String(std::string::String),
    None,
}

use Token::*;

impl From<std::string::String> for Token {
    fn from(tok: std::string::String) -> Self {
        if &tok == "[" {
            LeftBracket
        } else if &tok == "]" {
            RightBracket
        } else if &tok != "\"" && tok.starts_with('"') && tok.ends_with('"') {
            let s = tok
                .strip_prefix('"')
                .unwrap()
                .strip_suffix('"')
                .unwrap()
                .to_string();
            String(s)
        } else if let Ok(i) = tok.parse::<i64>() {
            I64(i)
        } else if let Ok(n) = tok.parse::<f64>() {
            F64(n)
        } else if tok.starts_with('\\') {
            let term = tok.strip_prefix("\\").unwrap().trim().to_string();
            if term.len() == 0 {
                None
            } else {
                DeferredTerm(term)
            }
        } else {
            Term(tok)
        }
    }
}

pub fn tokenize(line: &str) -> Vec<Token> {
    // TODO: Validate that a line does not contain unterminated strings.
    // TODO: Handle character escapes for quotes, newlines, etc. (But here?)
    let re: Regex = Regex::new(r#"(\[|\]|".*?"|[^\s\[\]]*)"#).unwrap();
    let line = line.replace('\n', " ");
    re.captures_iter(&line)
        .flat_map(|cap| cap.iter().take(1).collect::<Vec<_>>())
        .filter_map(|res| res.map(|mat| mat.as_str()))
        .take_while(|s| !s.starts_with('#'))
        .filter(|s| !s.is_empty())
        .map(|s| s.replace("\\n", "\n"))
        .map(Token::from)
        .collect()
}

#[test]
fn token_test() {
    let actual = "1 1 +";
    let expected = vec![I64(1), I64(1), Term("+".into())];

    assert_eq!(expected, tokenize(actual));
}

#[test]
fn token_test_2() {
    let actual = "\"hello\" \"there\"";
    let expected = vec![String("hello".into()), String("there".into())];

    assert_eq!(expected, tokenize(actual));
}

#[test]
fn token_test_3() {
    let actual = "\"hello there\"";
    let expected = vec![String("hello there".into())];

    assert_eq!(expected, tokenize(actual));
}

#[test]
fn token_test_4() {
    let actual = "\" hello there \"";
    let expected = vec![String(" hello there ".into())];

    assert_eq!(expected, tokenize(actual));
}

#[test]
fn token_test_5() {
    let actual = "1 2 \" hello three \" 4 5";
    let expected = vec![
        I64(1),
        I64(2),
        String(" hello three ".into()),
        I64(4),
        I64(5),
    ];

    assert_eq!(expected, tokenize(actual));
}

#[test]
fn token_test_6() {
    let actual = "1 2 \"a # in a string is fine\" #but at the end is ignored";
    let expected = vec![I64(1), I64(2), String("a # in a string is fine".into())];

    assert_eq!(expected, tokenize(actual));
}

#[test]
fn token_test_7() {
    let actual = "1 1 [ + ] call .s";
    let expected = vec![
        I64(1),
        I64(1),
        LeftBracket,
        Term("+".into()),
        RightBracket,
        Term("call".into()),
        Term(".s".into()),
    ];

    assert_eq!(expected, tokenize(actual));
}

#[test]
fn token_test_8() {
    let actual = "1 1 [+] call .s";
    let expected = vec![
        I64(1),
        I64(1),
        LeftBracket,
        Term("+".into()),
        RightBracket,
        Term("call".into()),
        Term(".s".into()),
    ];

    assert_eq!(expected, tokenize(actual));
}

#[test]
fn token_test_9() {
    let actual = "[1 1][+]doin .s";
    let expected = vec![
        LeftBracket,
        I64(1),
        I64(1),
        RightBracket,
        LeftBracket,
        Term("+".into()),
        RightBracket,
        Term("doin".into()),
        Term(".s".into()),
    ];

    assert_eq!(expected, tokenize(actual));
}

#[test]
fn token_test_10() {
    let actual = "1 \\dup do .s";
    let expected = vec![
        I64(1),
        DeferredTerm("dup".into()),
        Term("do".into()),
        Term(".s".into()),
    ];

    assert_eq!(expected, tokenize(actual));
}
