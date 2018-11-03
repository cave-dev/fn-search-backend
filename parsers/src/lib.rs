#[macro_use]
extern crate nom;
use nom::{multispace, is_space};

#[derive(Debug, PartialEq)]
struct Elm<'a> {
  module_name: &'a [u8],
  exposed: &'a [u8],
}

fn is_end_parenthesis(a: u8) -> bool {
    a == b')'
}

named!(elm_exposed,
    take_till!(is_end_parenthesis)
);

named!(elm<&[u8], Elm>,
    do_parse!(
        tag!("module") >>
        multispace >>
        module_name: take_till!(is_space) >>
        multispace >>
        tag!("exposing") >>
        multispace >>
        char!('(') >>
        exposed: elm_exposed >>
        char!(')') >>
        (Elm{module_name: module_name, exposed: exposed})
    )
);

#[test]
fn it_works() {
    let s0 = elm(&b"module Main exposing (..)"[..]);
    let s1 = elm(&b"module Main exposing (Test0)"[..]);
    let s2 = elm(&b"module Main exposing (Test0, Test1)"[..]);

    assert_eq!(s0, Ok((&b""[..], Elm { module_name: &b"Main"[..], exposed: &b".."[..] })));
    assert_eq!(s1, Ok((&b""[..], Elm { module_name: &b"Main"[..], exposed: &b"Test0"[..] })));
    assert_eq!(s2, Ok((&b""[..], Elm { module_name: &b"Main"[..], exposed: &b"Test0, Test1"[..] })));
}
