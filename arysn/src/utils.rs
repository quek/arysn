pub fn escape_like<T: AsRef<str>>(s: T) -> String {
    let mut result = String::new();
    for c in s.as_ref().chars() {
        match c {
            '%' => result.push_str(r"\%"),
            '_' => result.push_str(r"\_"),
            '\\' => result.push_str(r"\\"),
            c => result.push(c),
        }
    }
    result
}

pub fn title_case(s: &str) -> String {
    s.split("_")
        .map(|s| {
            let mut c = s.chars();
            match c.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<Vec<String>>()
        .join("")
}
