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

    // Collect workspaces
    let workspaces = if args.yes {
        // Non-interactive mode: single workspace with auto-detected defaults
        let current_dir = Path::new(".");
        let detected = detect::detect_project_type(current_dir);
        let project_name = detect::detect_project_name(current_dir);
        let project_type = detected.unwrap_or(templates::ProjectType::Other);
        let template = templates::ProjectTemplate::for_type(project_type);
        vec![template.to_workspace(project_name.as_deref())]
    } else {
        // Interactive mode: allow multiple workspaces
        collect_workspaces()?
    };

    // Generate config
    let config = Config { workspaces };

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

/// Collect multiple workspaces interactively
fn collect_workspaces() -> Result<Vec<Workspace>, Box<dyn Error>> {
    let mut workspaces: Vec<Workspace> = Vec::new();
    let mut workspace_num = 1;

    loop {
        println!("\n--- Workspace {} ---", workspace_num);

        // For first workspace, use current directory; for subsequent, prompt for directory
        let workspace_dir = if workspace_num == 1 {
            Path::new(".").to_path_buf()
        } else {
            prompt_workspace_directory()?
        };

        // Detect project type and name for the workspace directory
        let detected = detect::detect_project_type(&workspace_dir);
        let project_name = detect::detect_project_name(&workspace_dir);

        // Let user select/confirm project type
        let project_type = select_project_type(detected)?;
        let template = templates::ProjectTemplate::for_type(project_type);

        // Prompt for workspace configuration
        let workspace_dir_opt = if workspace_num == 1 {
            None // Use template default for first workspace
        } else {
            Some(workspace_dir.as_path())
        };
        let workspace = prompt_workspace_config(&template, project_name.as_deref(), workspace_dir_opt)?;
        workspaces.push(workspace);

        // Ask if user wants to add another workspace
        if !prompt_add_another()? {
            break;
        }
        workspace_num += 1;
    }

    Ok(workspaces)
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
    workspace_dir: Option<&Path>,
) -> Result<Workspace, Box<dyn Error>> {
    // Watch directory - use workspace_dir if provided, otherwise template default
    let default_dir = workspace_dir
        .and_then(|p| p.to_str())
        .unwrap_or(template.default_watch_dir);

    let dir = Text::new("Watch directory:")
        .with_default(default_dir)
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
        env_file: None,
    })
}

/// Prompt to add another workspace
fn prompt_add_another() -> Result<bool, Box<dyn Error>> {
    let add_another = Confirm::new("Add another workspace?")
        .with_default(false)
        .with_help_message("Configure additional directories to watch")
        .prompt()?;

    Ok(add_another)
}

/// Prompt for workspace directory path
fn prompt_workspace_directory() -> Result<std::path::PathBuf, Box<dyn Error>> {
    let dir = Text::new("Workspace directory path:")
        .with_default(".")
        .with_help_message("Path to the directory for this workspace (relative or absolute)")
        .prompt()?;

    Ok(std::path::PathBuf::from(dir))
}
