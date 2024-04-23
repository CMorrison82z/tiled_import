use ndarray::Array2;
use nom::{
    bytes::complete::*,
    character::complete::*,
    combinator::*,
    error::{ContextError, ErrorKind, ParseError},
    multi::*,
    sequence::*,
    IResult,
};

use crate::data_types::*;

pub(crate) fn get_tileset_for_gid(tilesets: &[TileSet], gid: Gid) -> Option<&TileSet> {
    let Gid(gid) = gid;

    tilesets
        .iter()
        .filter(|ts| ts.first_gid <= gid)
        .max_by_key(|ts| ts.first_gid)
}

fn whitespace<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    take_while(move |c| " \t\r\n".contains(c))(i)
}

fn u32_parse<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, u32, E> {
    // TODO:
    // Don't unwrap
    map(digit1, |s: &'a str| s.parse::<u32>().unwrap())(i)
}

pub(crate) fn csv_root<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, Vec<u32>, E> {
    cut(separated_list1(
        preceded(whitespace, char(',')),
        preceded(whitespace, u32_parse),
    ))(i)
}

pub(crate) fn parse_tiles_csv<'a>(i: &'a str) -> Result<Array2<u32>, nom::Err<(&str, ErrorKind)>> {
    csv_root::<(&str, ErrorKind)>(i).map(|(_, x)| {
        // TODO:
        // More robust way of determining the newline character
        // For now, just dividing by 2 (because the separator character `,` is a character)
        let columns = i.find('\n').unwrap() / 2;

        ndarray::Array2::from_shape_vec((columns, x.len() / columns ), x).unwrap()
    })
}
