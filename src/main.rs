mod error;
mod lrc;
mod vtt;

use clap::Parser;

use error::ProcessError;
use lrc::output_to_lrc_file;
use vtt::parse_vtt_blocks;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct CmdOpts {
    input_file_name: String,
    #[arg(short, long, default_value_t)]
    output_file_name: String,
}

fn process(opts: CmdOpts) -> Result<(), ProcessError> {
    let blocks = parse_vtt_blocks(&opts.input_file_name)?;
    output_to_lrc_file(&opts.output_file_name, &blocks)
}

fn main() {
    let mut opts = CmdOpts::parse();
    if opts.output_file_name.is_empty() {
        if let Some(base_name) = opts.input_file_name.strip_suffix(".vtt") {
            opts.output_file_name = format!("{}.lrc", base_name);
        } else {
            opts.output_file_name = format!("{}.lrc", opts.input_file_name);
        }
    }

    if let Err(error) = process(opts) {
        match error {
            ProcessError::Io(e) => eprintln!("{}", e),
            ProcessError::Parse(e) => eprintln!("{}", e),
        }
    }
}
