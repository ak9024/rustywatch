pub mod detect;
pub mod generator;
pub mod prompts;
pub mod templates;

use crate::args::InitArgs;
use std::error::Error;

pub fn run(args: InitArgs) -> Result<(), Box<dyn Error>> {
    prompts::interactive_init(args)
}
