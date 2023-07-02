use clap::Parser;

#[derive(Parser)]
#[command(
    author,
    version,
    about,
    long_about = "Capture DNS requests and show their QNames"
)]
pub struct Opt {
    #[arg(long, help = "device")]
    pub device: Option<String>,
    #[arg(
        long,
        help = "pcap filter",
        default_value = "ip proto \\udp and src port 53"
    )]
    pub filter: String,
    #[arg(long, help = "do not start web service")]
    pub noweb: bool,
}
