use regex::Regex;

pub fn tokenize(line: &str) -> Vec<String> {
    // TODO: Validate that a line does not contain unterminated strings.
    // TODO: Allow for string escapes for quotes, newlines, etc
    let re: Regex = Regex::new(r#"(\[|\]|".*?"|[^\s\[\]]*)"#).unwrap();
    let line = line.replace('\n', " ");
    re.captures_iter(&line)
        .flat_map(|cap| cap.iter().take(1).collect::<Vec<_>>())
        .filter_map(|res| res.map(|mat| mat.as_str()))
        .take_while(|s| !s.starts_with('#'))
        .filter(|s| !s.is_empty())
        .map(|s| s.replace("\\n", "\n"))
        .collect()
}

#[test]
pub fn token_test() {
    let actual = "1 1 +";
    let expected = vec!["1", "1", "+"];

    assert_eq!(expected, tokenize(actual));
}

#[test]
pub fn token_test_2() {
    let actual = "\"hello\" \"there\"";
    let expected = vec!["\"hello\"", "\"there\""];

    assert_eq!(expected, tokenize(actual));
}

#[test]
pub fn token_test_3() {
    let actual = "\"hello there\"";
    let expected = vec!["\"hello there\""];

    assert_eq!(expected, tokenize(actual));
}

#[test]
pub fn token_test_4() {
    let actual = "\" hello there \"";
    let expected = vec!["\" hello there \""];

    assert_eq!(expected, tokenize(actual));
}

#[test]
pub fn token_test_5() {
    let actual = "1 2 \" hello three \" 4 5";
    let expected = vec!["1", "2", "\" hello three \"", "4", "5"];

    assert_eq!(expected, tokenize(actual));
}

#[test]
pub fn token_test_6() {
    let actual = "1 2 \"a # in a string is fine\" #but at the end is ignored";
    let expected = vec!["1", "2", "\"a # in a string is fine\""];

    assert_eq!(expected, tokenize(actual));
}

#[test]
pub fn token_test_7() {
    let actual = "1 1 [ + ] call .s";
    let expected = vec!["1", "1", "[", "+", "]", "call", ".s"];

    assert_eq!(expected, tokenize(actual));
}

#[test]
pub fn token_test_8() {
    let actual = "1 1 [+] call .s";
    let expected = vec!["1", "1", "[", "+", "]", "call", ".s"];

    assert_eq!(expected, tokenize(actual));
}

#[test]
pub fn token_test_9() {
    let actual = "[1 1][+]doin .s";
    let expected = vec!["[", "1", "1", "]", "[", "+", "]", "doin", ".s"];

    assert_eq!(expected, tokenize(actual));
}
