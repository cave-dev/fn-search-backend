pub fn is_space_or_newline(c: char) -> bool {
    c.is_whitespace() || c == '\n'
}

pub fn is_alphanumeric(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

pub fn is_space_or_newline_or_comma(c: char) -> bool {
    is_space_or_newline(c) || c == ','
}

pub fn is_space_or_newline_or_(c: char) -> bool {
    is_space_or_newline(c) || c == ','
}
