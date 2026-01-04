use crate::args::InitArgs;
use crate::config::schema::{CommandType, Config, Workspace};

use super::{detect, generator, templates};
use inquire::{Confirm, Select, Text};
use std::error::Error;
use std::path::Path;

/// Main interactive initialization flow
pub fn interactive_init(args: InitArgs) -> Result<(), Box<dyn Error>> {
    let output_path = Path::new(&args.output);

    // Check for existing config
    if output_path.exists() && !args.force {
        let overwrite = Confirm::new(&format!("{} already exists. Overwrite?", args.output))
            .with_default(false)
            .prompt()?;

        if !overwrite {
            println!("Aborted.");
            return Ok(());
        }
    }

    // Auto-detect project type
    let current_dir = Path::new(".");
    let detected = detect::detect_project_type(current_dir);
    let project_name = detect::detect_project_name(current_dir);

    // Determine project type
    let project_type = if args.yes {
        detected.unwrap_or(templates::ProjectType::Other)
    } else {
        select_project_type(detected)?
    };

    // Get template defaults
    let template = templates::ProjectTemplate::for_type(project_type);

    // Collect user input (or use defaults with --yes)
    let workspace = if args.yes {
        template.to_workspace(project_name.as_deref())
    } else {
        prompt_workspace_config(&template, project_name.as_deref())?
    };

    // Generate config
    let config = Config {
        workspaces: vec![workspace],
    };

    // Show preview if interactive
    if !args.yes {
        println!("\n--- Configuration Preview ---");
        println!("{}", generator::preview_config(&config)?);
        println!("---");

        let confirm = Confirm::new("Write this configuration?")
            .with_default(true)
            .prompt()?;

        if !confirm {
            println!("Aborted.");
            return Ok(());
        }
    }

    // Write config file
    generator::write_config(&config, &args.output)?;

    println!("\nCreated {} successfully!", args.output);
    println!("Run 'rustywatch' to start watching for changes.");

    Ok(())
}

fn select_project_type(
    detected: Option<templates::ProjectType>,
) -> Result<templates::ProjectType, Box<dyn Error>> {
    let options: Vec<String> = templates::ProjectType::all()
        .iter()
        .map(|t| t.display_name().to_string())
        .collect();

    let default_idx = detected.map(|t| t as usize).unwrap_or(5);

    let prompt_msg = match detected {
        Some(t) => format!("Detected {} project. Select project type:", t.display_name()),
        None => "Select project type:".to_string(),
    };

    let selection = Select::new(&prompt_msg, options)
        .with_starting_cursor(default_idx)
        .prompt()?;

    Ok(match selection.as_str() {
        "Rust" => templates::ProjectType::Rust,
        "Node.js" => templates::ProjectType::NodeJS,
        "Go" => templates::ProjectType::Go,
        "Python" => templates::ProjectType::Python,
        "Bun" => templates::ProjectType::Bun,
        _ => templates::ProjectType::Other,
    })
}

fn prompt_workspace_config(
    template: &templates::ProjectTemplate,
    project_name: Option<&str>,
) -> Result<Workspace, Box<dyn Error>> {
    // Watch directory
    let dir = Text::new("Watch directory:")
        .with_default(template.default_watch_dir)
        .with_help_message("Directory to watch for file changes")
        .prompt()?;

    // Build command
    let cmd_default = template.default_commands.join("; ");
    let cmd = Text::new("Build/run command:")
        .with_default(&cmd_default)
        .with_help_message("Command(s) to execute when files change")
        .prompt()?;

    // Binary path (only for compiled languages)
    let bin_path = if template.default_bin_path.is_some() {
        let default = template
            .default_bin_path
            .map(|p| {
                if let Some(name) = project_name {
                    p.replace("{{project_name}}", name)
                } else {
                    p.replace("{{project_name}}", "app")
                }
            })
            .unwrap_or_default();

        let input = Text::new("Binary path (leave empty if not applicable):")
            .with_default(&default)
            .with_help_message("Path to the compiled binary to run after build")
            .prompt()?;

        if input.is_empty() {
            None
        } else {
            Some(input)
        }
    } else {
        None
    };

    // Ignore patterns
    let ignore_default = template.default_ignore.join(", ");
    let ignore_input = Text::new("Ignore patterns (comma-separated):")
        .with_default(&ignore_default)
        .with_help_message("File/directory patterns to ignore (e.g., target/, node_modules/)")
        .prompt()?;

    let ignore: Option<Vec<String>> = if ignore_input.is_empty() {
        None
    } else {
        Some(
            ignore_input
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
        )
    };

    Ok(Workspace {
        dir,
        cmd: CommandType::Single(cmd),
        ignore,
        bin_path,
        bin_arg: None,
    })
}
