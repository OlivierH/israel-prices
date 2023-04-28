pub fn sanitize_name(input: &str) -> String {
    let mut out = input;

    while out.starts_with('"') && out.ends_with('"') {
        out = out
            .strip_prefix('"')
            .unwrap()
            .strip_suffix('"')
            .unwrap()
            .trim();
    }
    if out.starts_with('"') && !out[1..].contains('"') {
        out = &out[1..].trim();
    }
    if out.starts_with("'") && !out[1..].contains("'") {
        out = &out[1..].trim();
    }
    while out.ends_with('!') {
        out = out.strip_suffix('!').unwrap().trim();
    }
    if out.starts_with("*מבצע*") {
        out = out.strip_prefix("*מבצע*").unwrap().trim();
    }
    while out.starts_with("*") && !out[1..].contains("*") {
        out = &out[1..];
    }
    let out_str = out.replace("\"\"", "ֿֿֿֿ\"");
    out = &out_str;

    out.trim().to_string()
}
