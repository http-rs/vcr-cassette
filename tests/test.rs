use std::fs;
use vcr_cassette::Cassette;

#[test]
fn smoke_json() {
    let names = ["example.json"];

    for name in names {
        let name = format!("tests/fixtures/{}", name);
        println!("testing: {}", name);
        let example = fs::read_to_string(name).unwrap();
        let _out: Cassette = serde_json::from_str(&example).unwrap();
    }
}

#[test]
fn smoke_yaml() {
    let names = [
        "example.yaml",
        "example2.yaml",
        "with_localhost_requests.yaml",
        "match_request_on.yaml",
    ];

    for name in names {
        let name = format!("tests/fixtures/{}", name);
        println!("testing: {}", name);
        let example = fs::read_to_string(name).unwrap();
        let _out: Cassette = serde_yaml::from_str(&example).unwrap();
    }
}
