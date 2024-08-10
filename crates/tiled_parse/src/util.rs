use ndarray::Array2;
use nom::{
    bytes::complete::*,
    character::complete::*,
    combinator::*,
    error::{ContextError, ErrorKind, ParseError},
    multi::*,
    number::complete::recognize_float,
    sequence::*,
    IResult,
};

use crate::data_types::*;

fn whitespace<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    take_while(move |c| " \t\r\n".contains(c))(i)
}

fn u32_parse<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, u32, E> {
    // TODO:
    // Don't unwrap
    map(digit1, |s: &'a str| s.parse::<u32>().unwrap())(i)
}

fn f32_parse<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, f32, E> {
    // TODO:
    // Don't unwrap
    map(recognize_float, |s: &'a str| s.parse::<f32>().unwrap())(i)
}

pub fn csv_root<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, Vec<u32>, E> {
    cut(separated_list1(
        preceded(whitespace, char(',')),
        preceded(whitespace, u32_parse),
    ))(i)
}

pub fn spaced_f32_pairs<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, Vec<PairF32>, E> {
    cut(separated_list1(
        whitespace,
        separated_pair(f32_parse, char(','), f32_parse),
    ))(i)
}

pub fn parse_spaced_f32_pairs<'a>(i: &'a str) -> Result<Vec<PairF32>, nom::Err<(&str, ErrorKind)>> {
    spaced_f32_pairs(i).map(|(_, v)| v)
}

pub fn parse_tiles_csv<'a>(i: &'a str) -> Result<Array2<u32>, nom::Err<(&str, ErrorKind)>> {
    let columns = i
        .lines()
        .next()
        .unwrap()
        .chars()
        .filter(|c| *c == ',')
        .count();

    csv_root::<(&str, ErrorKind)>(i).map(|(_, x)| {
        // TODO:
        // Make this better...
        let mut rr = ndarray::Array2::from_shape_vec((x.len() / columns, columns), x)
            .unwrap()
            .reversed_axes();
        rr.invert_axis(ndarray::Axis(1));
        rr
    })
}
