use nom::{
    character::complete::*,
    combinator::*,
    error::{ContextError, ErrorKind, ParseError},
    multi::*,
    number::complete::recognize_float,
    sequence::*,
    IResult,
};

use crate::types::*;

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

pub(crate) fn csv_root<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, Vec<u32>, E> {
    cut(separated_list1(
        preceded(multispace0, char(',')),
        preceded(multispace0, u32_parse),
    ))(i)
}

pub(crate) fn spaced_f32_pairs<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, Vec<PairF32>, E> {
    cut(separated_list0(
        multispace1,
        separated_pair(f32_parse, char(','), f32_parse),
    ))(i)
}

pub(crate) fn parse_spaced_f32_pairs<'a>(
    i: &'a str,
) -> Result<Vec<PairF32>, nom::Err<(&'a str, ErrorKind)>> {
    spaced_f32_pairs(i).map(|(_, v)| v)
}
