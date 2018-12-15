#[macro_use]
extern crate nom;

mod helpers;
use helpers::{
    is_alphanumeric,
    is_space_or_newline,
    is_space_or_newline_or_comma,
};

mod structs;
use structs::{
    ElmModule,
    TypeOrFunction,
    Type,
    Function,
    ElmCode,
};

named!(pub expose_all<&str, ElmModule>,
    map!(tag!(".."), |_| ElmModule::All)
);

named!(pub expose_functions_and_types<&str, ElmModule>,
    map!(
        separated_list!(tag!(","), function_or_type),
        ElmModule::List
    )
);

named!(pub function_or_type<&str, TypeOrFunction>,
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

named!(pub multi_spaces_or_new_line_or_comma<&str, &str>,
    map!(take_while!(is_space_or_newline_or_comma), |s| s)
);

named!(pub ignore_any<&str, ElmCode>,
    map!(take!(1), |_| ElmCode::Ignore)
);

named!(pub ignore_comments<&str, ElmCode>,
    map!(
        alt!(
            preceded!(tag!("{-"), take_until_and_consume!("-}")) |
            preceded!(tag!("--"), take_until_and_consume!("\n"))
        ),
        |_| ElmCode::Comment
    )
);

/*
    separate by -> ignore spaces, tabs, newline
        name : type -> type -> type
        name : type -> (type, type) -> type
*/
named!(pub function<&str, ElmCode>,
    map!(
        do_parse!(
            name: take_while!(is_alphanumeric) >>
            multi_spaces_or_new_line_or_comma >>
            char!(':') >>
            multi_spaces_or_new_line_or_comma >>
            types: take_until!(name) >>
            tag!(name) >>
            (name, types)
        ),
        |(name, types)| ElmCode::Function
    )
);

named!(pub elm_mod_def<&str, ElmModule>,
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

named!(elm<&str, (ElmModule, Vec<ElmCode>)>,
    do_parse!(
        exposed: elm_mod_def >>
        // defs: alt!(type_def | type_def_function | ignore_comments | code_to_be_ignored) >>
        multi_spaces_or_new_line_or_comma >>
        defs: many1!(alt!(ignore_comments | function | complete!(ignore_any))) >>
        // defs: opt!(many0!(ignore_any)) >>
        (exposed, defs)
    )
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multiline_comment() {
        assert_eq!(
            ignore_comments("{- \nhello world \n-}"),
            Ok(("",
                ElmCode::Comment
            ))
        );
    }

    #[test]
    fn singleline_comment() {
        assert_eq!(
            ignore_comments("-- hello world\nhello"),
            Ok(("hello",
                ElmCode::Comment
            ))
        );
    }

    #[test]
    fn ignore_all() {
        named!(test<&str, Vec<ElmCode>>, many1!(complete!(ignore_any)));
        assert_eq!(
            test("t s "),
            Ok(("", vec!(ElmCode::Ignore, ElmCode::Ignore, ElmCode::Ignore, ElmCode::Ignore)))
        );
    }

    #[test]
    fn function_type_signature() {
        assert_eq!(
            function("test : hello -> \nworld -> Int\ntest"),
            Ok(("",
                ElmCode::Function
            ))
        );
    }

    #[test]
    fn expose_all_works() {
        assert_eq!(
            elm_mod_def("module Main exposing (..)"),
            Ok(("", ElmModule::All))
        );
    }

    #[test]
    fn expose_one_type_works() {
        assert_eq!(
            elm_mod_def("module Main exposing (test0)"),
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
    fn expose_many_types_works() {
        assert_eq!(
            elm_mod_def("module Main exposing (Test0, test1)"),
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
            elm_mod_def("module Utils.Time\n   exposing\n  ( a\n , b\n , c\n , d\n   )"),

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
