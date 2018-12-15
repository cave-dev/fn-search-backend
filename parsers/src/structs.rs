#[derive(Debug, PartialEq)]

pub enum ElmCode {
    Comment,
    Declaration,
    Ignore,
    Function,
    Type,
}

#[derive(Debug, PartialEq)]
pub enum ElmModule<'a> {
    All,
    List(Vec<TypeOrFunction<'a>>),
}

type Name<'a> = &'a str;
type Definition<'a> = &'a str;
type TypeSignature <'a> = &'a str;

#[derive(Debug, PartialEq)]
pub enum TypeOrFunction<'a> {
    Type(Type<'a>),
    Function(Function<'a>),
}

#[derive(Debug, PartialEq)]
pub struct Type<'a> {
    pub name: Name<'a>,
    pub definition: Option<Definition<'a>>,
}

#[derive(Debug, PartialEq)]
pub struct Function<'a> {
    pub name: Name<'a>,
    pub type_signature: Option<TypeSignature<'a>>,
}
