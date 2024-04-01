use std::collections::HashMap;

use crate::parse;

use super::parse::*;

#[cfg(test)]

#[test]
fn parse_tmx() {
    let data = std::fs::read("HangerV1.tmx").unwrap();

    let output = parse(std::str::from_utf8(&data).unwrap());

    panic!("{:#?}", output);
}
