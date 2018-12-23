use crate::helpers::{
    is_alphanumeric,
    is_space_or_newline,
    is_space_or_newline_or_comma,
    is_allowed_for_types_and_functions,
};

use crate::structs::{
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
            many1!(
                do_parse!(
                    many0!(comments) >>
                    take_while!(is_space_or_newline) >>
                    ()
                )
            ),
            alt!(
                do_parse!(
                    s: take_while!(is_alphanumeric) >>
                    tag!("(") >>
                    take_while!(is_allowed_for_types_and_functions) >>
                    tag!(")") >>
                    (s)
                ) |
                take_while!(is_alphanumeric)
            ),
            many1!(
                do_parse!(
                    many0!(comments) >>
                    take_while!(is_space_or_newline) >>
                    ()
                )
            )
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

named!(pub comments<&str, &str>,
    map!(
        alt!(
            preceded!(tag!("{-"), take_until_and_consume!("-}")) |
            preceded!(tag!("--"), take_until_and_consume!("\n"))
        ),
        |s| s
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
            tag!("\n") >>
            name: take_while!(is_alphanumeric) >>
            multi_spaces_or_new_line_or_comma >>
            char!(':') >>
            multi_spaces_or_new_line_or_comma >>
            types: take_until!(name) >>
            tag!(name) >>
            (name, types)
        ),
        |(name, types)| {
            let type_signature =
                types
                .split("->")
                .map(|s| s.replace("\n", ""))
                .map(|s| s.replace("\t", ""))
                .map(|s| s.trim().to_string())
                .collect::<Vec<String>>()
                ;

            ElmCode::Function(
                Function{
                    name: name,
                    type_signature: Some(type_signature)
                }
            )
        }
    )
);

// fails when module appears in comments before module statements
named!(pub elm_mod_def<&str, ElmModule>,
    do_parse!(
        take_until!("module") >>
        tag!("module") >>
        take_until!("exposing") >>
        tag!("exposing") >>
        multi_spaces_or_new_line_or_comma >>
        char!('(') >>
        exposed: alt!(expose_all | expose_functions_and_types) >>
        char!(')') >>
        (exposed)
    )
);

named!(pub elm<&str, (ElmModule, Vec<ElmCode>)>,
    alt!(
        complete!(
            do_parse!(
                exposed: elm_mod_def >>
                defs: many0!(
                        alt!(
                            complete!(ignore_comments) |
                            complete!(function) |
                            complete!(ignore_any)
                        )
                ) >>
                (
                    exposed,
                    defs.into_iter()
                        .filter(|w| w != &ElmCode::Ignore)
                        .collect::<Vec<ElmCode>>()
                )
            )
        )
        |
        complete!(
            do_parse!(
                defs: many0!(alt!(
                        complete!(ignore_comments) |
                        complete!(function) |
                        complete!(ignore_any)
                    )) >>
                (
                    ElmModule::List(vec!()),
                    defs.into_iter()
                        .filter(|w| w != &ElmCode::Ignore).collect::<Vec<ElmCode>>()
                )
            )
        )
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
            Ok(("",
                vec!(
                    ElmCode::Ignore,
                    ElmCode::Ignore,
                    ElmCode::Ignore,
                    ElmCode::Ignore
                )
            ))
        );
    }

    #[test]
    fn function_type_signature() {
        assert_eq!(
            function("\ntest : Int -> List Int -> \nInt\ntest"),
            Ok(("",
                ElmCode::Function(
                    Function{
                        name: "test",
                        type_signature:
                        Some(vec!(
                            "Int".to_string(),
                            "List Int".to_string(),
                            "Int".to_string()
                        )
                        )
                    }
                )
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

    #[test]
    fn integration() {
        assert_eq!(
            elm("module Utils exposing (test)\ntest : Int -> List Int -> Int\ntest"),
            Ok(("",
                (ElmModule::List(
                    vec!(
                        TypeOrFunction::Function(
                            Function{
                                name: "test",
                                type_signature: None
                            }
                        )
                    )
                ),
                vec!(
                    ElmCode::Function(
                        Function{
                            name: "test",
                            type_signature: Some(
                                vec!(
                                    "Int".to_string(),
                                    "List Int".to_string(),
                                    "Int".to_string()
                                )
                            )
                        }
                    )
                ))
            ))
        );
    }

    use std::fs;


    #[test]
    fn file_integration() {
        let contents = fs::read_to_string("Main.elm")
            .expect("Something went wrong reading the file");

        assert_eq!(
            elm(&contents),
            Ok(("",
                (ElmModule::All,
                 vec!(
                     ElmCode::Function(
                         Function{
                             name: "subscriptions",
                             type_signature: Some(
                                 vec!(
                                     "Model".to_string(),
                                     "Sub Msg".to_string(),
                                 )
                             )
                         }
                     ),
                     ElmCode::Function(
                         Function{
                             name: "init",
                             type_signature: Some(
                                 vec!(
                                     "Int".to_string(),
                                     "( Model, Cmd Msg )".to_string(),
                                 )
                             )
                         }
                     ),
                     ElmCode::Function(
                         Function{
                             name: "update",
                             type_signature: Some(
                                 vec!(
                                     "Msg".to_string(),
                                     "Model".to_string(),
                                     "( Model, Cmd Msg )".to_string(),
                                 )
                             )
                         }
                     ),
                     ElmCode::Function(
                         Function{
                             name: "functionView",
                             type_signature: Some(
                                 vec!(
                                     "SearchResult".to_string(),
                                     "Html Msg".to_string(),
                                 )
                             )
                         }
                     ),
                     ElmCode::Function(
                         Function{
                             name: "view",
                             type_signature: Some(
                                 vec!(
                                     "Model".to_string(),
                                     "Html Msg".to_string(),
                                 )
                             )
                         }
                     ),
                     ElmCode::Function(
                         Function{
                             name: "searchResultDecoder",
                             type_signature: Some(
                                 vec!(
                                     "Decode.Decoder (List SearchResult)".to_string(),
                                 )
                             )
                         }
                     ),
                     ElmCode::Function(
                         Function{
                             name: "repoDecoder",
                             type_signature: Some(
                                 vec!(
                                     "Decode.Decoder SearchResultRepo".to_string(),
                                 )
                             )
                         }
                     ),
                     ElmCode::Function(
                         Function{
                             name: "resDecoder",
                             type_signature: Some(
                                 vec!(
                                     "Decode.Decoder SearchResultFn".to_string(),
                                 )
                             )
                         }
                     ),
                 ))
            ))
        );
    }
}
