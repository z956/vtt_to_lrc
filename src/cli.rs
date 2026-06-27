pub struct CmdOpts {
    pub help: bool,
    pub input_file_name: String,
    pub output_file_name: String,
}

pub fn parse_opts() -> Result<CmdOpts, String> {
    let mut opts = CmdOpts {
        help: false,
        input_file_name: String::new(),
        output_file_name: String::new(),
    };

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "-h" || arg == "--help" {
            opts.help = true;
            break;
        }
        if arg == "-o" || arg == "--output" {
            if !opts.output_file_name.is_empty() {
                return Err(String::from(
                    "Specifying multiple output file name is not supported",
                ));
            }
            match args.next() {
                None => {
                    return Err(format!("Missing output file name after option '{}'", arg));
                }
                Some(output_file_name) => {
                    if output_file_name.is_empty() {
                        return Err(String::from("Output file name should be non-empty"));
                    }
                    opts.output_file_name = output_file_name;
                }
            }
        } else {
            if !opts.input_file_name.is_empty() {
                return Err(String::from(
                    "Specifying multiple input file name is not supported",
                ));
            }
            opts.input_file_name = arg;
        }
    }

    if opts.input_file_name.is_empty() && !opts.help {
        return Err(String::from("Error: No input file name specified"));
    }

    if !opts.help && opts.output_file_name.is_empty() {
        if let Some(base_name) = opts.input_file_name.strip_suffix(".vtt") {
            opts.output_file_name = format!("{}.lrc", base_name);
        } else {
            opts.output_file_name = format!("{}.lrc", opts.input_file_name);
        }
    }

    Ok(opts)
}

pub fn print_help() {
    println!("Usage: vtt_to_lrc [options...]");
    println!("  -h, --help                      Show help information");
    println!("  -o, --output <output_file_name> Output lrc file name");
    println!("  <file_name>                     Input vtt file name");
}
