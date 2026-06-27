mod cli;
mod error;
mod lrc;
mod vtt;

use error::ProcessError;
use lrc::output_to_lrc_file;
use vtt::parse_vtt_blocks;

fn process(opts: cli::CmdOpts) -> Result<(), ProcessError> {
    let blocks = parse_vtt_blocks(&opts.input_file_name)?;
    output_to_lrc_file(&opts.output_file_name, &blocks)
}

fn main() {
    let opt = cli::parse_opts();
    match opt {
        Err(error) => eprintln!("{}", error),
        Ok(opts) => {
            if opts.help {
                cli::print_help();
            } else {
                if let Err(error) = process(opts) {
                    match error {
                        ProcessError::Io(e) => eprintln!("{}", e),
                        ProcessError::Parse(e) => eprintln!("{}", e),
                    }
                }
            }
        }
    }
}
