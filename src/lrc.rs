use std::fs::File;
use std::io::{ErrorKind, Write};

use crate::error::ProcessError;
use crate::vtt::{TimeStamp, VttBlock};

pub fn output_to_lrc_file(file_name: &str, blocks: &[VttBlock]) -> Result<(), ProcessError> {
    match File::create_new(file_name) {
        Ok(mut file) => write_blocks_to_file(&mut file, blocks),
        Err(error) => {
            if error.kind() == ErrorKind::AlreadyExists {
                // TODO: ask user that file already exists. Ask if we should overwrite it
                print!(
                    "File {} already exists. Do you want to overwrite it? [Y/N] ",
                    file_name
                );
                std::io::stdout().flush()?;

                let overwrite = loop {
                    let mut answer = String::new();
                    std::io::stdin().read_line(&mut answer)?;
                    match answer.trim() {
                        "Y" | "y" => {
                            break true;
                        }
                        "N" | "n" => {
                            break false;
                        }
                        _ => {
                            print!("Do you want to overwrite it? [Y/N]");
                            std::io::stdout().flush()?;
                        }
                    }
                };
                if !overwrite {
                    println!("Canceled");
                    return Ok(());
                }

                let mut file = File::create(file_name)?;
                write_blocks_to_file(&mut file, blocks)
            } else {
                Err(error.into())
            }
        }
    }
}

fn format_lrc_timestamp(timestamp: &TimeStamp, min_digits: usize) -> String {
    // TODO
    // format: [mm:ss.xx], where xx is 1/100 second
    // the min_digits controls how many digits we need to present the min
    let min = timestamp.min + timestamp.hour * 60;
    let mut centisecs = timestamp.milli / 10;
    while centisecs >= 100 {
        centisecs /= 10;
    }

    // format it
    format!(
        "[{:0width$}:{:02}.{:02}]",
        min,
        timestamp.sec,
        centisecs,
        width = min_digits
    )
}

fn get_min_digits(timestamp: &TimeStamp) -> usize {
    let mut min_digits = 2;
    let mut min = timestamp.min;
    if timestamp.hour > 0 {
        min += timestamp.hour * 60;
        min_digits = 0;
        while min != 0 {
            min_digits += 1;
            min /= 10;
        }
    }
    min_digits
}

fn write_blocks_to_file(file: &mut File, blocks: &[VttBlock]) -> Result<(), ProcessError> {
    // last one of block should have largest value
    if blocks.is_empty() {
        return Ok(());
    }
    let min_digits = get_min_digits(&blocks[blocks.len() - 1].timestamp.start);
    for block in blocks {
        let timestamp = format_lrc_timestamp(&block.timestamp.start, min_digits);
        for data in &block.data {
            writeln!(file, "{}{}", timestamp, data)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_min_digits() {
        let timestamp = TimeStamp {
            hour: 0,
            min: 1,
            sec: 2,
            milli: 3,
        };
        let digits = get_min_digits(&timestamp);
        assert_eq!(digits, 2);
    }

    #[test]
    fn test_get_min_digits_has_hour() {
        let timestamp = TimeStamp {
            hour: 1,
            min: 1,
            sec: 2,
            milli: 3,
        };
        let digits = get_min_digits(&timestamp);
        assert_eq!(digits, 2);
    }

    #[test]
    fn test_get_min_digits_has_2_hours() {
        let timestamp = TimeStamp {
            hour: 2,
            min: 1,
            sec: 2,
            milli: 3,
        };
        let digits = get_min_digits(&timestamp);
        assert_eq!(digits, 3);
    }

    #[test]
    fn test_format_lrc_timestamp() {
        let timestamp = TimeStamp {
            hour: 0,
            min: 1,
            sec: 2,
            milli: 3,
        };
        let digits = get_min_digits(&timestamp);
        let result = format_lrc_timestamp(&timestamp, digits);
        assert_eq!(result, "[01:02.00]");
    }

    #[test]
    fn test_format_lrc_timestamp_2_hours() {
        let timestamp = TimeStamp {
            hour: 2,
            min: 1,
            sec: 2,
            milli: 3,
        };
        let digits = get_min_digits(&timestamp);
        let result = format_lrc_timestamp(&timestamp, digits);
        assert_eq!(result, "[121:02.00]");
    }

    #[test]
    fn test_format_lrc_timestamp_trim_milli() {
        let timestamp = TimeStamp {
            hour: 0,
            min: 1,
            sec: 2,
            milli: 987,
        };
        let digits = get_min_digits(&timestamp);
        let result = format_lrc_timestamp(&timestamp, digits);
        assert_eq!(result, "[01:02.98]");
    }
}
