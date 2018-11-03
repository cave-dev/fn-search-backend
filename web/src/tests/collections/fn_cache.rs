
use collections::fn_cache::FnCache;
use fn_search_backend_db::models::Function;
use std::collections::HashSet;

#[test]
fn build_cache_for_fn_ref() {
    let v: Vec<&Function> = Vec::new();
    let _: FnCache = v.into_iter().collect();
}

#[test]
fn build_cache_for_fn() {
    let v: Vec<Function> = Vec::new();
    let _: FnCache = v.into_iter().collect();
}

lazy_static! {
    static ref TEST_FNS: [Function; 6] = [
        Function{
            id: 0,
            repo_id: 0,
            name: String::from("derpyfn"),
            type_signature: String::from("Int -> Int"),
        },
        Function{
            id: 1,
            repo_id: 0,
            name: String::from("whatever"),
            type_signature: String::from("String -> Int"),
        },
        Function{
            id: 2,
            repo_id: 1,
            name: String::from("lol"),
            type_signature: String::from("Int -> Bool"),
        },
        Function{
            id: 3,
            repo_id: 1,
            name: String::from("zxc"),
            type_signature: String::from("Bool -> Bool"),
        },
        Function{
            id: 4,
            repo_id: 1,
            name: String::from("fef"),
            type_signature: String::from("Int -> String"),
        },
        Function{
            id: 5,
            repo_id: 0,
            name: String::from("rer"),
            type_signature: String::from("String -> Int"),
        },
    ];
}

fn setup_test_cache() -> FnCache {
    let mut v: Vec<&Function> = Vec::new();
    for f in TEST_FNS.iter() {
        v.push(f);
    }
    v.into_iter().collect()
}

#[test]
fn build_works_for_functions() {
    setup_test_cache();
}

#[test]
fn search_gives_values() {
    let c = setup_test_cache();
    let res = c.search("String -> Int", 10, None);
    assert!(res.is_some());
    let res = res.unwrap();
    assert_eq!(res.len(), 2);
    let vec_of_ref: Vec<&Function> = res.iter().collect();
    assert_eq!(vec_of_ref.as_slice(), &[&TEST_FNS[1], &TEST_FNS[5]]);
}

#[test]
fn search_gives_value() {
    let c = setup_test_cache();
    let res = c.search("Bool -> Bool", 10, None);
    assert!(res.is_some());
    let res = res.unwrap();
    assert_eq!(res.len(), 1);
    let vec_of_ref: Vec<&Function> = res.iter().collect();
    assert_eq!(vec_of_ref.as_slice(), &[&TEST_FNS[3]]);
}

#[test]
fn search_gives_none_on_invalid_start() {
    let c = setup_test_cache();
    let res = c.search("String -> Int", 10, Some(2));
    assert!(res.is_none());
}

#[test]
fn search_gives_some_start_index() {
    let c = setup_test_cache();
    let res = c.search("String -> Int", 10, Some(1));
    assert!(res.is_some());
    let res = res.unwrap();
    assert_eq!(res.len(), 1);
    let vec_of_ref: Vec<&Function> = res.iter().collect();
    assert_eq!(vec_of_ref.as_slice(), &[&TEST_FNS[5]]);
}

#[test]
fn search_gives_only_num() {
    let c = setup_test_cache();
    let res = c.search("String -> Int", 1, None);
    assert!(res.is_some());
    let res = res.unwrap();
    assert_eq!(res.len(), 1);
    let vec_of_ref: Vec<&Function> = res.iter().collect();
    assert_eq!(vec_of_ref.as_slice(), &[&TEST_FNS[1]]);
}

#[test]
fn search_gives_only_num_with_start() {
    let c = setup_test_cache();
    let res = c.search("String -> Int", 1, Some(1));
    assert!(res.is_some());
    let res = res.unwrap();
    assert_eq!(res.len(), 1);
    let vec_of_ref: Vec<&Function> = res.iter().collect();
    assert_eq!(vec_of_ref.as_slice(), &[&TEST_FNS[5]]);
}

#[test]
fn search_gives_none_on_bad_key() {
    let c = setup_test_cache();
    let res = c.search("String -> In", 10, None);
    assert!(res.is_none());
}

#[test]
fn suggest_gives_suggestion() {
    let c = setup_test_cache();
    let res = c.suggest("String -> In", 10);
    assert!(res.is_some());
    let res = res.unwrap();
    assert_eq!(res.len(), 1);
    assert_eq!(res, vec!["String -> Int"]);
}

#[test]
fn suggest_gives_suggestions() {
    let c = setup_test_cache();
    let res = c.suggest("In", 10);
    assert!(res.is_some());
    let res = res.unwrap();
    assert_eq!(res.len(), 3);
    println!("{:?}", res);
    let sig_map: HashSet<&str> = res.into_iter().collect();
    let expected: HashSet<&str> = vec![
        "Int -> String",
        "Int -> Bool",
        "Int -> Int"
    ].into_iter().collect();
    assert_eq!(sig_map, expected);
}

#[test]
fn suggest_gives_x_suggestions() {
    let c = setup_test_cache();
    let res = c.suggest("In", 2);
    assert!(res.is_some());
    let res = res.unwrap();
    assert_eq!(res.len(), 2);
}

#[test]
fn suggest_gives_none() {
    let c = setup_test_cache();
    let res = c.suggest("Ink", 10);
    assert!(res.is_none());
}
