use clap::Parser;

#[derive(Parser)]
#[clap(name = "Osori")]
#[clap(author = "webaresoft <webaresoft@webaresoft.com>")]
#[clap(version = "1.0")]
#[clap(about = "Osori is Awesome API Gateway", long_about = None)]
struct Args {
    #[clap(short='a', long)]
    domain_address: Option<String>,

    #[clap(short='n', long)]
    engine_name: Option<String>,

    #[clap(short='g', long)]
    group_name: Option<String>,

    #[clap(short='t', long)]
    health_check: Option<usize>,

    #[clap(short='s', long)]
    signal: Option<String>,

    #[clap(short='v', long, value_name="Sleep")]
    version: Option<String>,

    #[clap(short='V', long)]
    verbose_mode: Option<bool>,
}

pub fn parse_args() {
    let args = Args::parse();
    if let Some(domain_address) = args.domain_address {
        println!("Args list {}", domain_address);
    }
}

