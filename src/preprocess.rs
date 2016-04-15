use unescape::unescape;

pub fn clean_string(mut input: String) -> String {
    input = String::from(input.trim());
    input = unescape(&input).unwrap_or(String::from(""));

    if input == "" {
        return input;
    }

    // remove line-comments
    let mut last = '\0';
    let mut len = input.len();
    for (i, c) in input.chars().enumerate() {
        if last == '/' && c == '/' {
            len = i-1;
            break;
        }
        last = c;
    }

    input.truncate(len);

    input
}

pub fn add_semicolon(mut input: String) -> String {
    if !input.ends_with(';') && !input.ends_with("*/") && !input.ends_with('}') {
        input.push_str(";")
    }
    input
}
