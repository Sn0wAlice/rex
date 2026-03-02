use clap::{Parser, Subcommand};
use anyhow::Result;

extern crate rex;
use rex::com::{ssl, file, net, reg, domain, diskinfo, carve, hash};

#[derive(Parser)]
#[command(
    name = "rex",
    about = r"
           __    Hello i'am Rex
          / _) /
   .-^^^-/ /
  /       /
<_.|_|-|_|

Rex - Security CLI toolkit for SOC analysts and blue teams",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// SSL certificate operations
    Ssl {
        #[command(subcommand)]
        command: SslCommands,
    },
    /// File analysis operations
    File {
        #[command(subcommand)]
        command: FileCommands,
    },
    /// Network operations
    Net {
        #[command(subcommand)]
        command: NetCommands,
    },
    /// Registry / systemd log analysis
    Reg {
        #[command(subcommand)]
        command: RegCommands,
    },
    /// Domain analysis operations
    Domain {
        #[command(subcommand)]
        command: DomainCommands,
    },
    /// Identify hash type from a hash string
    Hash {
        #[command(subcommand)]
        command: HashCommands,
    },
    /// Display disk information
    Diskinfo,
    /// Carve (recover) files from a disk image
    Carve {
        /// Path to the device or image file
        image: String,
        /// Recover all files (no limit)
        #[arg(long)]
        all: bool,
        /// Only recover deleted files (skip first 512KB)
        #[arg(long)]
        only_deleted: bool,
        /// Base output directory for recovered files
        #[arg(long, default_value = "recovered")]
        output: String,
    },
}

#[derive(Subcommand)]
enum SslCommands {
    /// Dump SSL certificates for a domain
    Dump {
        /// Target domain
        domain: String,
    },
}

#[derive(Subcommand)]
enum FileCommands {
    /// PDF file operations
    Pdf {
        #[command(subcommand)]
        command: PdfCommands,
    },
}

#[derive(Subcommand)]
enum PdfCommands {
    /// Extract metadata, images, and scripts from a PDF
    Extract {
        /// Path to the PDF file
        path: String,
        /// Output directory for extracted content
        #[arg(long, default_value = "output/file/pdf/extracted")]
        output: String,
    },
}

#[derive(Subcommand)]
enum NetCommands {
    /// Capture and log network traffic (requires root)
    Log,
}

#[derive(Subcommand)]
enum RegCommands {
    /// Systemd journal analysis
    Systemd {
        #[command(subcommand)]
        command: SystemdCommands,
    },
}

#[derive(Subcommand)]
enum SystemdCommands {
    /// Extract logs from the systemd journal
    Extract {
        /// Number of log entries
        #[arg(short, default_value_t = 100)]
        n: usize,
        /// Group logs by syslog identifier
        #[arg(long)]
        group: bool,
    },
    /// Scan journal for security-relevant events
    Scan {
        /// Number of log entries to scan
        #[arg(short, default_value_t = 100)]
        n: usize,
    },
    /// Deep scan for suspicious activity
    Deepscan {
        /// Save results to a timestamped file
        #[arg(long)]
        save: bool,
    },
    /// Extract failed SSH login attempts
    Sshfail,
}

#[derive(Subcommand)]
enum DomainCommands {
    /// Mail configuration analysis (SPF, DKIM, DMARC)
    Mail {
        #[command(subcommand)]
        command: MailCommands,
    },
    /// Typosquatting domain generation
    Typosquat {
        /// Target domain
        domain: String,
        /// Output file path
        #[arg(long)]
        output: Option<String>,
        /// Comma-separated list of methods
        #[arg(long)]
        method: Option<String>,
    },
}

#[derive(Subcommand)]
enum MailCommands {
    /// Scan mail DNS records (SPF, DKIM, DMARC)
    Scan {
        /// Target domain
        domain: String,
    },
}

#[derive(Subcommand)]
enum HashCommands {
    /// Detect the type of a hash (interactive if no hash provided)
    Detect {
        /// Hash string to identify (omit for interactive input)
        hash: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Ssl { command } => match command {
            SslCommands::Dump { domain } => ssl::dump(&domain).await?,
        },
        Commands::File { command } => match command {
            FileCommands::Pdf { command } => match command {
                PdfCommands::Extract { path, output } => file::extract_pdf(&path, &output)?,
            },
        },
        Commands::Net { command } => match command {
            NetCommands::Log => net::logs_network()?,
        },
        Commands::Reg { command } => match command {
            RegCommands::Systemd { command } => match command {
                SystemdCommands::Extract { n, group } => {
                    if group {
                        reg::sysd_extract_group(n)?;
                    } else {
                        reg::sysd_extract(n)?;
                    }
                }
                SystemdCommands::Scan { n } => reg::sysd_scan(n)?,
                SystemdCommands::Deepscan { save } => reg::sysd_deepscan(save)?,
                SystemdCommands::Sshfail => reg::sshfail()?,
            },
        },
        Commands::Domain { command } => match command {
            DomainCommands::Mail { command } => match command {
                MailCommands::Scan { domain } => domain::mail_scan(&domain).await?,
            },
            DomainCommands::Typosquat { domain, output, method } => {
                domain::typosquat(&domain, output.as_deref(), method.as_deref())?;
            }
        },
        Commands::Hash { command } => match command {
            HashCommands::Detect { hash: h } => hash::detect(h.as_deref())?,
        },
        Commands::Diskinfo => diskinfo::run()?,
        Commands::Carve { image, all, only_deleted, output } => {
            carve::run(&image, all, only_deleted, &output)?;
        }
    }

    Ok(())
}
