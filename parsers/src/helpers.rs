pub fn is_space_or_newline(c: char) -> bool {
    c.is_whitespace() || c == '\n'
}

pub fn is_alphanumeric(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

pub fn is_operator(c: char) -> bool {
    ['+', '-', '/', '*', '^', '=', '>', '<', '&', '|'].iter().any(|&x| x == c)
}

pub fn is_allowed_for_types_and_functions(c: char) -> bool {
    is_alphanumeric(c) || ['.', ','].iter().any(|&x| x == c)
}

pub fn is_space_or_newline_or_comma(c: char) -> bool {
    is_space_or_newline(c) || c == ','
}
