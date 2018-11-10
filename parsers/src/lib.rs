#[macro_use]
extern crate nom;

use nom::types::CompleteStr;
use nom::eof;

#[derive(Debug, PartialEq)]
enum ElmModule<'a> {
    All,
    List(Vec<TypeOrFunction<'a>>),
}

type Name<'a> = &'a str;
type Definition<'a> = &'a str;
type TypeSignature <'a> = &'a str;

#[derive(Debug, PartialEq)]
enum TypeOrFunction<'a> {
    Type(Type<'a>),
    Function(Function<'a>),
}

#[derive(Debug, PartialEq)]
struct Type<'a> {
    name: Name<'a>,
    definition: Option<Definition<'a>>,
}

#[derive(Debug, PartialEq)]
struct Function<'a> {
    name: Name<'a>,
    type_signature: Option<TypeSignature<'a>>,
}

fn is_space_or_newline(c: char) -> bool {
    c.is_whitespace() || c == '\n'
}

fn is_alphanumeric(c: char) -> bool {
    c.is_alphanumeric()
}

fn is_space_or_newline_or_comma(c: char) -> bool {
    is_space_or_newline(c) || c == ','
}

named!(expose_all<&str, ElmModule>,
    map!(tag!(".."), |_| ElmModule::All)
);

named!(expose_functions_and_types<&str, ElmModule>,
    map!(
        separated_list!(tag!(","), function_or_type),
        ElmModule::List
    )
);

named!(function_or_type<&str, TypeOrFunction>,
    map!(
        delimited!(
            take_while!(is_space_or_newline),
            take_while!(is_alphanumeric),
            take_while!(is_space_or_newline)
        ),
        |s| {
            // based on the assumption that anything starting with:
            //      lowercase is a function
            //      uppcase is a type
            if s.chars().next().unwrap().is_lowercase() {
                TypeOrFunction::Function(
                    Function{
                        name: s,
                        type_signature: None
                    }
                )
            } else {
                TypeOrFunction::Type(
                    Type{
                        name: s,
                        definition: None
                    }
                )
            }
        }
    )
);

named!(multi_spaces_or_new_line_or_comma<&str, &str>,
    map!(take_while!(is_space_or_newline_or_comma), |s| s)
);

named!(elm<&str, ElmModule>,
    do_parse!(
        tag!("module") >>
        multi_spaces_or_new_line_or_comma >>
        take_till!(is_space_or_newline_or_comma) >>
        multi_spaces_or_new_line_or_comma >>
        tag!("exposing") >>
        multi_spaces_or_new_line_or_comma >>
        char!('(') >>
        exposed: alt!(expose_all | expose_functions_and_types) >>
        char!(')') >>
        (exposed)
    )
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expose_all_works() {
        assert_eq!(
            elm("module Main exposing (..)"),
            Ok(("",
                ElmModule::All
            ))
        );
    }

    #[test]
    fn expose_one_type_works() {
        assert_eq!(
            elm("module Main exposing (test0)"),
            Ok(("",
                ElmModule::List(
                    vec!(
                        TypeOrFunction::Function(
                            Function{name: "test0", type_signature: None}
                        ),
                    )
                )
            ))
        );
    }

    #[test]
    fn test_my_expectations() {
        // named!(f<&str, TypeOrFunction>, alt!(function | type_));
        // named!(g<&[u8], Vec<&[u8]>>, separated_list!(tag!(","), is_not!("")));

        // assert_eq!(
            // g(&b"(Test0)"[..]),
            // Ok((&b""[..], vec!()))
        // );

        // assert_eq!(
            // function("Test0"),
            // // Ok(("", ElmModule::All))
            // Ok(("", TypeOrFunction::Function( Function{name: "Test0", type_signature: None})))
        // );

        named!(s<&str, &str>, ws!(take_while!(is_alphanumeric)));

        assert_eq!(
            s(" Test0\0"),
            // Ok(("", ElmModule::All))
            Ok(("\u{0}", "Test0"))
        );
    }

    #[test]
    fn expose_many_types_works() {
        assert_eq!(
            elm("module Main exposing (Test0, test1)"),
            Ok(("",
                ElmModule::List(
                    vec!(
                        TypeOrFunction::Type(
                            Type{name: "Test0", definition: None}
                        ),
                        TypeOrFunction::Function(
                            Function{name: "test1", type_signature: None}
                        ),
                    )
                )
            ))
        );
    }

    #[test]
    fn newline_separator() {
        assert_eq!(
            elm("module Utils.Time\n   exposing\n  ( a\n , b\n , c\n , d\n   )"),

            Ok(("",
                ElmModule::List(
                    vec!(
                        TypeOrFunction::Function(
                            Function{name: "a", type_signature: None}
                        ),
                        TypeOrFunction::Function(
                            Function{name: "b", type_signature: None}
                        ),
                        TypeOrFunction::Function(
                            Function{name: "c", type_signature: None}
                        ),
                        TypeOrFunction::Function(
                            Function{name: "d", type_signature: None}
                        ),
                    )
                )
            ))
        );
    }
}
