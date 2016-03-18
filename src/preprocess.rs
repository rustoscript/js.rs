pub fn clean_string(input: String) -> Option<String> {
    if input == "" {
        return None;
    }

    // insert semicolon if necessary
    //if !input.ends_with(";") && !input.ends_with("}") {
    //    input.push_str(";");
    //}

    // ignore comments
    if input.starts_with("//"){
        return None;
    }

    Some(input)
}
