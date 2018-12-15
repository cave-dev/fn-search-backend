#[macro_use]
extern crate nom;

pub(crate) mod helpers;
pub(crate) mod structs;
pub(crate) mod parser;

use hashbrown::HashSet;
use std::error::Error;
use crate::structs::{ElmCode, ElmModule, TypeOrFunction, Type, Function};
use crate::parser::elm;

#[derive(Debug)]
pub enum ElmExport {
    Function{
        name: String,
        type_signature: Option<Vec<String>>,
    },
    Type{
        name: String,
        definition: String,
    },
}

#[derive(Debug)]
pub struct ElmExports {
    exports: Vec<ElmExport>,
}

impl ElmExports {
    fn new() -> ElmExports {
        ElmExports {
            exports: vec![],
        }
    }
}

pub fn get_elm_exports(code: &str) -> Result<ElmExports, ()> {
    let (_, (module, elm_code)) = match elm(code) {
        Ok(v) => v,
        Err(_) => return Err(()),
    };
    if let ElmModule::List(l) = module {
        Ok(exports_from_module_list(l.as_ref(), elm_code.as_ref()))
    } else {
        Ok(exports_from_module_all(elm_code.as_ref()))
    }
}

fn exports_from_module_list(l: &[TypeOrFunction], elm_code: &[ElmCode]) -> ElmExports {
    let mut exports = ElmExports::new();
    // get a set containing all types & functions that will be exported and we care about
    let to_export: HashSet<&str> = l
        .iter()
        .map(|export| {
            match export {
                TypeOrFunction::Type(Type{ref name, ..}) => {
                    *name
                },
                TypeOrFunction::Function(Function{ref name, ..}) => {
                    *name
                },
            }
        })
        .collect();
    // collect functions and types that are defined in the module exports
    for exp in l.iter() {
        match exp {
            TypeOrFunction::Type(Type{ref name, definition: Some(def)}) => {
                exports.exports.push(
                    ElmExport::Type{
                        name: String::from(*name),
                        definition: String::from(*def),
                    }
                )
            },
            TypeOrFunction::Function(Function{ref name, type_signature: Some(sig)}) => {
                exports.exports.push(
                    ElmExport::Function{
                        name: String::from(*name),
                        type_signature: Some(sig.clone()),
                    }
                )
            },
            // ignore if there is not an inline definition
            _ => {},
        }
    }
    // collect functions and types from code
    for code_bit in elm_code.iter() {
        match code_bit {
            ElmCode::Type(Type{ref name, ref definition}) => {
                if to_export.contains(*name) {
                    exports.exports.push(
                        ElmExport::Type{
                            name: String::from(*name),
                            definition: definition
                                .map(|def| String::from(def))
                                .unwrap_or_else(|| String::new()),
                        }
                    )
                }
            },
            ElmCode::Function(Function{ref name, ref type_signature}) => {
                if to_export.contains(*name) {
                    exports.exports.push(
                        ElmExport::Function{
                            name: String::from(*name),
                            type_signature: type_signature.clone(),
                        }
                    )
                }
            },
            // do nothing
            _ => {},
        }
    }
    exports
}

fn exports_from_module_all(elm_code: &[ElmCode]) -> ElmExports {
    let mut exports = ElmExports::new();
    // collect functions and types from code
    for code_bit in elm_code.iter() {
        match code_bit {
            ElmCode::Type(Type{ref name, ref definition}) => {
                exports.exports.push(
                    ElmExport::Type{
                        name: String::from(*name),
                        definition: definition
                            .map(|def| String::from(def))
                            .unwrap_or_else(|| String::new()),
                    }
                )
            },
            ElmCode::Function(Function{ref name, ref type_signature}) => {
                exports.exports.push(
                    ElmExport::Function{
                        name: String::from(*name),
                        type_signature: type_signature.clone(),
                    }
                )
            },
            // do nothing
            _ => {},
        }
    }
    exports
}
