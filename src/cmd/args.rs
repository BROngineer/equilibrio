use clap::Parser;

#[derive(Parser)]
#[command(version = "1.0", author = "Artem Bortnikov <brongineer747@gmail.com>")]
pub struct Args {
    #[arg(short, long)]
    pub address: String,
    
    #[arg(short, long)]
    pub port: u16,
    
    #[arg(short, long)]
    pub endpoint: String,
}
