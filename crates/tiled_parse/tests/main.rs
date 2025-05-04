use tiled_parse::parse::*;

#[test]
fn parse_tmx_csv() {
    let data = std::fs::read("HangerV1.tmx").unwrap();

    assert!(parse_embedded(std::str::from_utf8(&data).unwrap()).is_ok());
}

#[test]
fn parse_tmx_base64() {
    let data = std::fs::read("HangerV1_b64.tmx").unwrap();

    assert!(parse_embedded(std::str::from_utf8(&data).unwrap()).is_ok());
}
