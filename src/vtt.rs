use std::fs::File;
use std::io::{BufRead, BufReader};

use crate::error::ProcessError;

pub struct TimeStamp {
    pub hour: u32,
    pub min: u32,
    pub sec: u32,
    pub milli: u32,
}

impl TimeStamp {
    pub fn parse(text: &str, line_num: usize) -> Result<Self, ProcessError> {
        // format: hh:mm:ss.mmm
        if !text.is_ascii() {
            return Err(ProcessError::Parse(format!(
                "Invalid timestamp format at line {}: Timestamp line must be in ASCII format",
                line_num
            )));
        }
        if text.len() != 12 {
            return Err(ProcessError::Parse(format!(
                "Invalid timestamp format at line {}: Start timestamp len is invalid",
                line_num
            )));
        }
        Self::check_separator(text, ':', 2, line_num)?;
        Self::check_separator(text, ':', 5, line_num)?;
        Self::check_separator(text, '.', 8, line_num)?;

        let hour = Self::get_num(&text[0..2], line_num)?;
        let min = Self::get_num(&text[3..5], line_num)?;
        let sec = Self::get_num(&text[6..8], line_num)?;
        let milli = Self::get_num(&text[9..], line_num)?;
        if hour != 0 {
            eprintln!(
                "Warning: Detect non-zero hour value at line {}. Output LRC file could be incorrect",
                line_num
            );
        }

        Ok(Self {
            hour,
            min,
            sec,
            milli,
        })
    }

    fn get_num(text: &str, line_num: usize) -> Result<u32, ProcessError> {
        text.parse::<u32>().map_err(|e| {
            ProcessError::Parse(format!(
                "Invalid timestamp format at line {}: Failed to parse text '{}' to integer: {}",
                line_num, text, e
            ))
        })
    }

    fn check_separator(
        text: &str,
        sep: char,
        pos: usize,
        line_num: usize,
    ) -> Result<(), ProcessError> {
        match text.chars().nth(pos) {
            None => Err(ProcessError::Parse(format!(
                "Invalid timestamp format at line {}: Failed to get separator at position {}",
                line_num, pos
            ))),
            Some(result) => {
                if result != sep {
                    return Err(ProcessError::Parse(format!(
                        "Invalid timestamp format at line {}: Unexpected separator at position {}. Expect '{}'. Got '{}'.",
                        line_num, pos, sep, result
                    )));
                }
                Ok(())
            }
        }
    }
}

pub struct TimeStampRange {
    pub start: TimeStamp,
    #[allow(dead_code)]
    pub end: TimeStamp,
}

impl TimeStampRange {
    pub fn parse(line: &str, line_num: usize) -> Result<Self, ProcessError> {
        // format: hh:mm:ss.mmm --> hh:mm:ss.mmm
        let tokens: Vec<&str> = line.split_whitespace().collect();
        if tokens.len() != 3 {
            return Err(ProcessError::Parse(format!(
                "Invalid timestamp format at line {}: Token number is not 3",
                line_num
            )));
        }
        if tokens[1] != "-->" {
            return Err(ProcessError::Parse(format!(
                "Invalid timestamp format at line {}: Invalid split token",
                line_num
            )));
        }

        Ok(TimeStampRange {
            start: TimeStamp::parse(tokens[0], line_num)?,
            end: TimeStamp::parse(tokens[2], line_num)?,
        })
    }
}

pub struct VttBlock {
    pub timestamp: TimeStampRange,
    pub data: Vec<String>,
}

impl VttBlock {
    pub fn parse<I>(lines: &mut I, line_num: &mut usize) -> Result<Option<Self>, ProcessError>
    where
        I: Iterator<Item = Result<String, std::io::Error>>,
    {
        // NOTE: a block may end with a empty line, or not (end of file)
        // vtt block:
        // hh:mm:ss.mmm --> hh:mm:ss.mmm
        // line1
        // line2
        // <empty line>
        // When EOF, empty line may not exist

        // timestamp
        let timestamp_line = loop {
            match lines.next() {
                Some(line_result) => {
                    *line_num += 1;
                    let line = line_result?;
                    if !line.is_empty() {
                        break line;
                    }
                }
                None => return Ok(None),
            }
        };
        let timestamp = TimeStampRange::parse(&timestamp_line, *line_num)?;

        // lines
        let mut data: Vec<String> = Vec::new();
        loop {
            match lines.next() {
                Some(line_result) => {
                    let line = line_result?;
                    *line_num += 1;
                    if line.is_empty() {
                        // read end of block
                        break;
                    }
                    data.push(line);
                    if data.len() > 1 {
                        eprintln!(
                            "Warning: Detect multiple data in a vtt block at line {}. Output LRC file could not be correct",
                            *line_num
                        );
                    }
                }
                None => {
                    // read EOF
                    break;
                }
            }
        }

        if data.is_empty() {
            return Err(ProcessError::Parse(format!(
                "Invalid VTT block at line {}: only timestamp line found",
                *line_num
            )));
        }
        Ok(Some(Self { timestamp, data }))
    }
}

fn parse_vtt_blocks_from_reader<R>(reader: R) -> Result<Vec<VttBlock>, ProcessError>
where
    R: BufRead,
{
    let mut line_num = 0;
    let mut lines = reader.lines();
    let header = next_line(&mut lines, &mut line_num)?;
    if header != "WEBVTT" {
        return Err(ProcessError::Parse(format!(
            "Invalid header at line {}: Expect 'WEBVTT'. Get '{}'",
            line_num, header
        )));
    }
    let empty = next_line(&mut lines, &mut line_num)?;
    if !empty.is_empty() {
        return Err(ProcessError::Parse(String::from(
            "No empty line found at line 2",
        )));
    }

    let mut blocks: Vec<VttBlock> = Vec::new();
    while let Some(vtt_block) = VttBlock::parse(&mut lines, &mut line_num)? {
        blocks.push(vtt_block);
    }

    Ok(blocks)
}

pub fn parse_vtt_blocks(file_name: &str) -> Result<Vec<VttBlock>, ProcessError> {
    let file = File::open(file_name)?;
    let reader = BufReader::new(file);
    parse_vtt_blocks_from_reader(reader)
}

fn next_line<I>(lines: &mut I, line_num: &mut usize) -> Result<String, ProcessError>
where
    I: Iterator<Item = Result<String, std::io::Error>>,
{
    match lines.next() {
        Some(line_result) => {
            let line = line_result?;
            *line_num += 1;
            Ok(line)
        }
        None => Err(ProcessError::Parse(String::from("Unexpected EOF"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn assert_timestamp(timestamp: &TimeStamp, hour: u32, min: u32, sec: u32, milli: u32) {
        assert_eq!(timestamp.hour, hour);
        assert_eq!(timestamp.min, min);
        assert_eq!(timestamp.sec, sec);
        assert_eq!(timestamp.milli, milli);
    }

    #[test]
    fn test_parse_valid_timestamp() {
        let timestamp = TimeStamp::parse("01:02:03.456", 1).unwrap();
        assert_timestamp(&timestamp, 1, 2, 3, 456);
    }

    #[test]
    fn test_non_ascii_timestamp() {
        let result = TimeStamp::parse("あ01:02:03.456", 1);
        assert!(result.is_err());
    }

    #[test]
    fn test_timestamp_invalid_digit() {
        let result = TimeStamp::parse("001:02:03.456", 1);
        assert!(result.is_err());
    }

    #[test]
    fn test_timestamp_invalid_sep() {
        let result = TimeStamp::parse("01a02:03.456", 1);
        assert!(result.is_err());
    }

    #[test]
    fn test_timestamp_invalid_text() {
        let result = TimeStamp::parse("123456789012", 1);
        assert!(result.is_err());
    }

    #[test]
    fn test_timestamp_range() {
        let range = TimeStampRange::parse("01:02:03.456 --> 02:03:04.456", 1).unwrap();
        assert_timestamp(&range.start, 1, 2, 3, 456);
        assert_timestamp(&range.end, 2, 3, 4, 456);
    }

    #[test]
    fn test_timestamp_range_extra_spaces() {
        let range = TimeStampRange::parse("   01:02:03.456   -->  02:03:04.456   ", 1).unwrap();
        assert_timestamp(&range.start, 1, 2, 3, 456);
        assert_timestamp(&range.end, 2, 3, 4, 456);
    }

    #[test]
    fn test_vttblock_parse() {
        let input: Vec<Result<String, std::io::Error>> = vec![
            Ok(String::from("01:02:03.456  --> 02:03:04.456")),
            Ok(String::from("line1")),
            Ok(String::from("")),
        ];
        let mut lines = input.into_iter();
        let mut line_num = 0;
        let vtt_block = VttBlock::parse(&mut lines, &mut line_num).unwrap().unwrap();
        assert_timestamp(&vtt_block.timestamp.start, 1, 2, 3, 456);
        assert_timestamp(&vtt_block.timestamp.end, 2, 3, 4, 456);

        assert_eq!(vtt_block.data.len(), 1);
        assert_eq!(vtt_block.data[0], "line1");
    }

    #[test]
    fn test_vttblock_parse_multi_lines() {
        let input: Vec<Result<String, std::io::Error>> = vec![
            Ok(String::from("01:02:03.456  --> 02:03:04.456")),
            Ok(String::from("line1")),
            Ok(String::from("line2")),
            Ok(String::from("")),
        ];
        let mut lines = input.into_iter();
        let mut line_num = 0;
        let vtt_block = VttBlock::parse(&mut lines, &mut line_num).unwrap().unwrap();

        assert_eq!(vtt_block.data.len(), 2);
        assert_eq!(vtt_block.data[0], "line1");
        assert_eq!(vtt_block.data[1], "line2");
    }

    #[test]
    fn test_vttblock_parse_no_empty_line() {
        let input: Vec<Result<String, std::io::Error>> = vec![
            Ok(String::from("01:02:03.456  --> 02:03:04.456")),
            Ok(String::from("line1")),
        ];
        let mut lines = input.into_iter();
        let mut line_num = 0;
        let vtt_block = VttBlock::parse(&mut lines, &mut line_num).unwrap().unwrap();

        assert_eq!(vtt_block.data.len(), 1);
        assert_eq!(vtt_block.data[0], "line1");
    }

    #[test]
    fn test_vttblock_parse_no_timestamp() {
        let input: Vec<Result<String, std::io::Error>> =
            vec![Ok(String::from("line1")), Ok(String::from(""))];
        let mut lines = input.into_iter();
        let mut line_num = 0;
        let result = VttBlock::parse(&mut lines, &mut line_num);

        assert!(result.is_err());
    }

    #[test]
    fn test_vttblock_parse_empty_line() {
        let input: Vec<Result<String, std::io::Error>> = vec![Ok(String::from(""))];
        let mut lines = input.into_iter();
        let mut line_num = 0;
        let result = VttBlock::parse(&mut lines, &mut line_num).unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn test_parse_vttblock() {
        let text = "\
WEBVTT

01:02:03.456 --> 02:03:04.456
line1
";
        let reader = Cursor::new(text);
        let blocks = parse_vtt_blocks_from_reader(reader).unwrap();
        assert_eq!(blocks.len(), 1);
        assert_timestamp(&blocks[0].timestamp.start, 1, 2, 3, 456);
        assert_timestamp(&blocks[0].timestamp.end, 2, 3, 4, 456);

        assert_eq!(blocks[0].data.len(), 1);
        assert_eq!(blocks[0].data[0], "line1");
    }

    #[test]
    fn test_parse_vttblock_invalid_header1() {
        let text = "\
01:02:03.456 --> 02:03:04.456
line1
";
        let reader = Cursor::new(text);
        let result = parse_vtt_blocks_from_reader(reader);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_vttblock_invalid_header2() {
        let text = "\
WEBVTT
01:02:03.456 --> 02:03:04.456
line1
";
        let reader = Cursor::new(text);
        let result = parse_vtt_blocks_from_reader(reader);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_vttblock_two_blocks() {
        let text = "\
WEBVTT

01:02:03.456 --> 02:03:04.456
line1

01:02:03.456 --> 02:03:04.456
line2
line3
";
        let reader = Cursor::new(text);
        let blocks = parse_vtt_blocks_from_reader(reader).unwrap();
        assert_eq!(blocks.len(), 2);
        assert_timestamp(&blocks[0].timestamp.start, 1, 2, 3, 456);
        assert_timestamp(&blocks[0].timestamp.end, 2, 3, 4, 456);
        assert_eq!(blocks[0].data.len(), 1);
        assert_eq!(blocks[0].data[0], "line1");

        assert_timestamp(&blocks[1].timestamp.start, 1, 2, 3, 456);
        assert_timestamp(&blocks[1].timestamp.end, 2, 3, 4, 456);
        assert_eq!(blocks[1].data.len(), 2);
        assert_eq!(blocks[1].data[0], "line2");
        assert_eq!(blocks[1].data[1], "line3");
    }

    #[test]
    fn test_parse_vttblock_two_blocks_multi_spaces() {
        let text = "\
WEBVTT

01:02:03.456 --> 02:03:04.456
line1


01:02:03.456 --> 02:03:04.456
line2
line3
";
        let reader = Cursor::new(text);
        let blocks = parse_vtt_blocks_from_reader(reader).unwrap();
        assert_eq!(blocks.len(), 2);
        assert_timestamp(&blocks[0].timestamp.start, 1, 2, 3, 456);
        assert_timestamp(&blocks[0].timestamp.end, 2, 3, 4, 456);

        assert_eq!(blocks[0].data.len(), 1);
        assert_eq!(blocks[0].data[0], "line1");

        assert_timestamp(&blocks[1].timestamp.start, 1, 2, 3, 456);
        assert_timestamp(&blocks[1].timestamp.end, 2, 3, 4, 456);

        assert_eq!(blocks[1].data.len(), 2);
        assert_eq!(blocks[1].data[0], "line2");
        assert_eq!(blocks[1].data[1], "line3");
    }

    #[test]
    fn test_parse_vttblock_end_with_empty_line() {
        let text = "\
WEBVTT

01:02:03.456 --> 02:03:04.456
line1

";
        let reader = Cursor::new(text);
        let blocks = parse_vtt_blocks_from_reader(reader).unwrap();
        assert_eq!(blocks.len(), 1);
        assert_timestamp(&blocks[0].timestamp.start, 1, 2, 3, 456);
        assert_timestamp(&blocks[0].timestamp.end, 2, 3, 4, 456);

        assert_eq!(blocks[0].data.len(), 1);
        assert_eq!(blocks[0].data[0], "line1");
    }
}
