#[macro_use]
extern crate nom;

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
    c == ' ' || c == '\n'
}

fn is_space_or_newline_or_comma(c: char) -> bool {
    c == ' ' || c == '\n' || c == ','
}

named!(expose_all<&str, ElmModule>,
    map!(tag!(".."), |_| ElmModule::All)
);

named!(expose_functions_and_types<&str, ElmModule>,
    map!(
        alt!(separated_list!(tag!(","), alt!(function | type_))
             | many1!(alt!(function | type_))
        ), ElmModule::List
    )
);

named!(function<&str, TypeOrFunction>,
    map!(
        delimited!(
            take_while!(is_space_or_newline),
            alt!(take_till!(is_space_or_newline_or_comma) | is_a!("")),
            take_while!(is_space_or_newline)
        ),
        |s: &str| {
            TypeOrFunction::Function(
                Function{
                    name: s,
                    type_signature: None
                })
        }
    )
);

named!(type_<&str, TypeOrFunction>,
    map!(take_till!(is_space_or_newline_or_comma),
        |s: &str| TypeOrFunction::Type(Type{name: s, definition: None})
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
            elm("module Main exposing (Test0)"),
            Ok(("",
                ElmModule::List(
                    vec!(
                        TypeOrFunction::Function(
                            Function{name: "Test0", type_signature: None}
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

        assert_eq!(
            function("Test0"),
            // Ok(("", ElmModule::All))
            Ok(("", TypeOrFunction::Function( Function{name: "Test0", type_signature: None})))
        );
    }

    #[test]
    fn expose_many_types_works() {
        assert_eq!(
            elm("module Main exposing (Test0, Test1)"),
            Ok(("",
                ElmModule::List(
                    vec!(
                        TypeOrFunction::Function(
                            Function{name: "Test0", type_signature: None}
                        ),
                        TypeOrFunction::Function(
                            Function{name: "Test1", type_signature: None}
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
