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
