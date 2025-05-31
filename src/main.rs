mod camera;
mod event_loop;
mod math;
mod matrix;
mod mesh;
mod pipeline;
mod render;

use clap::{Args, Parser, Subcommand};

// setup command line args

#[derive(Parser)]
pub struct CliArgs {
  #[clap(subcommand)]
  pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
  Graph,
  MeltingGraph,
  WaveEquation,
  Image(ImageArgs),
  CustomTexture,
  WaveEquationTexture,
}

#[derive(Debug, Args)]
pub struct ImageArgs {
  #[clap(long, required=true)]
  path: String,
}

// program entrypoint

fn main() {
  let args = CliArgs::parse();
  pollster::block_on(event_loop::run(args));
}
