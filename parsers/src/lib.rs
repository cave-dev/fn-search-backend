#[macro_use]
extern crate nom;

#[derive(Debug, PartialEq)]
enum ElmExpose<'a> {
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

fn is_space(s: char) -> bool {
    s == ' '
}

named!(expose_all<&str, ElmExpose>,
    map!(tag!(".."), |_| ElmExpose::All)
);

named!(function<&str, TypeOrFunction>,
    map!(delimited!(char!(' '), take_till!(is_space), char!(' ')),
        |s: &str| TypeOrFunction::Function(Function{name: s, type_signature: None})
    )
);

named!(type_<&str, TypeOrFunction>,
    map!(take_till!(is_space),
        |s: &str| TypeOrFunction::Type(Type{name: s, definition: None})
    )
);

named!(expose_functions_and_types<&str, ElmExpose>,
    map!(separated_nonempty_list!(tag!(","), alt!(function | type_)), ElmExpose::List)
);

named!(multispaces<&str, &str>,
    map!(is_a!(" "), |s| s)
);

named!(elm<&str, ElmExpose>,
    do_parse!(
        tag!("module") >>
        multispaces >>
        take_till!(is_space) >>
        multispaces >>
        tag!("exposing") >>
        multispaces >>
        char!('(') >>
        exposed: alt!(expose_all | expose_functions_and_types) >>
        char!(')') >>
        (exposed)
    )
);

#[test]
fn expose_all_works() {
    assert_eq!(
        elm("module Main exposing (..)"),
        Ok(("",
            ElmExpose::All
        ))
    );
}

#[test]
fn expose_one_type_works() {
    assert_eq!(
        elm("module Main exposing (Test0)"),
        Ok(("",
            ElmExpose::List(
                vec!(TypeOrFunction::Function(Function{name: "Test0", type_signature: None}))
            )
        ))
    );
}

#[test]
fn expose_many_types_works() {
    assert_eq!(
        elm("module Main exposing ( Test0, Test1 )"),
        Ok(("",
            ElmExpose::List(
                vec!(
                    TypeOrFunction::Function(Function{name: "Test0", type_signature: None}),
                    TypeOrFunction::Function(Function{name: "Test1", type_signature: None})
                )
            )
        ))
    );
}
