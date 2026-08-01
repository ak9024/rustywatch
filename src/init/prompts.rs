use crate::args::InitArgs;
use crate::config::schema::{Config, Workspace};
use crate::watch::filter::CompiledFilter;

use super::detect::{self, DetectedProject};
use super::{generator, templates};
use inquire::validator::Validation;
use inquire::{Confirm, MultiSelect, Select, Text};
use std::error::Error;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use templates::{ProjectTemplate, ProjectType};

/// Main initialization flow.
///
/// Scans `args.dir` for projects, turns each into a workspace, and writes the
/// result to `args.output`. With `--yes` everything detected is accepted as-is;
/// otherwise the user picks which projects to include and may customize each.
pub fn interactive_init(args: InitArgs) -> Result<(), Box<dyn Error>> {
    let root = PathBuf::from(&args.dir);
    if !root.is_dir() {
        return Err(format!("`{}` is not a directory", args.dir).into());
    }

    let interactive = !args.yes;
    if interactive && !std::io::stdin().is_terminal() {
        return Err(
            "`rustywatch init` needs a terminal; re-run with --yes to accept detected defaults"
                .into(),
        );
    }

    if !confirm_output_path(&args, interactive)? {
        println!("Aborted.");
        return Ok(());
    }

    let detected = detect::scan_projects(&root, args.depth);
    report_detection(&detected, &root, interactive);

    let workspaces = if interactive {
        collect_workspaces(&root, &detected)?
    } else {
        default_workspaces(&root, &detected)
    };

    let config = Config::new(workspaces);
    // Fail here rather than writing a config `rustywatch` will reject.
    config.validate()?;

    let yaml = generator::preview_config(&config)?;

    if args.dry_run {
        print!("{yaml}");
        return Ok(());
    }

    if interactive {
        println!("\n--- Configuration Preview ---");
        println!("{yaml}");
        println!("---");

        let confirm = Confirm::new("Write this configuration?")
            .with_default(true)
            .prompt()?;

        if !confirm {
            println!("Aborted.");
            return Ok(());
        }
    }

    generator::write_config(&config, &args.output)?;

    println!(
        "\nCreated {} with {} workspace(s).",
        args.output,
        config.workspaces.len()
    );
    println!("Run 'rustywatch' to start watching for changes.");

    Ok(())
}

/// Returns `Ok(false)` when the user declined to overwrite an existing config.
fn confirm_output_path(args: &InitArgs, interactive: bool) -> Result<bool, Box<dyn Error>> {
    if args.dry_run || args.force || !Path::new(&args.output).exists() {
        return Ok(true);
    }

    if !interactive {
        // `--yes` must never block on a prompt.
        return Err(format!("{} already exists; pass --force to overwrite", args.output).into());
    }

    Ok(
        Confirm::new(&format!("{} already exists. Overwrite?", args.output))
            .with_default(false)
            .prompt()?,
    )
}

fn report_detection(detected: &[DetectedProject], root: &Path, interactive: bool) {
    if detected.is_empty() {
        println!("\nNo known project markers found under {}.", root.display());
        if !interactive {
            println!("Writing a placeholder workspace — edit the command before running.");
        }
        return;
    }

    println!("\nDetected {} project(s):", detected.len());
    for project in detected {
        println!("  - {project}");
    }
}

/// The path written into the config for a project discovered at `relative`.
fn config_dir_for(relative: &Path) -> String {
    if relative.as_os_str().is_empty() {
        ".".to_string()
    } else {
        relative.to_string_lossy().replace('\\', "/")
    }
}

/// Builds the workspace a detected project maps to with no user input.
fn workspace_from_detected(root: &Path, project: &DetectedProject) -> Workspace {
    ProjectTemplate::for_type(project.project_type).to_workspace(
        &config_dir_for(&project.dir),
        &root.join(&project.dir),
        project.name.as_deref(),
    )
}

/// The workspaces `--yes` produces: everything detected, or a single
/// placeholder rooted at the scan directory.
fn default_workspaces(root: &Path, detected: &[DetectedProject]) -> Vec<Workspace> {
    if detected.is_empty() {
        let name = detect::detect_project_name(root);
        return vec![ProjectTemplate::for_type(ProjectType::Other).to_workspace(
            ".",
            root,
            name.as_deref(),
        )];
    }

    detected
        .iter()
        .map(|project| workspace_from_detected(root, project))
        .collect()
}

/// Collect workspaces interactively
fn collect_workspaces(
    root: &Path,
    detected: &[DetectedProject],
) -> Result<Vec<Workspace>, Box<dyn Error>> {
    let mut workspaces: Vec<Workspace> = Vec::new();

    if !detected.is_empty() {
        let chosen = MultiSelect::new("Which projects should RustyWatch watch?", detected.to_vec())
            .with_all_selected_by_default()
            .with_help_message("↑↓ move, space toggles, enter confirms")
            .prompt()?;

        let customize = !chosen.is_empty()
            && Confirm::new("Customize the generated settings?")
                .with_default(false)
                .with_help_message(
                    "No = keep the detected command, binary path and ignore patterns",
                )
                .prompt()?;

        for project in &chosen {
            let base = workspace_from_detected(root, project);
            workspaces.push(if customize {
                customize_workspace(root, project.project_type, base)?
            } else {
                base
            });
        }
    }

    // Nothing detected, or everything deselected: fall back to manual entry.
    if workspaces.is_empty() {
        workspaces.push(prompt_manual_workspace(root)?);
    }

    while prompt_add_another()? {
        workspaces.push(prompt_manual_workspace(root)?);
    }

    Ok(workspaces)
}

/// Ask for a directory, detect what is in it, then customize.
fn prompt_manual_workspace(root: &Path) -> Result<Workspace, Box<dyn Error>> {
    let dir = prompt_directory(root, ".")?;
    let fs_dir = root.join(&dir);

    let detected_type = detect::detect_project_type(&fs_dir);
    let project_type = select_project_type(detected_type)?;
    let name = detect::detect_project_name(&fs_dir);

    let base = ProjectTemplate::for_type(project_type).to_workspace(&dir, &fs_dir, name.as_deref());

    customize_workspace(root, project_type, base)
}

fn select_project_type(detected: Option<ProjectType>) -> Result<ProjectType, Box<dyn Error>> {
    let options = ProjectType::all();
    let starting_cursor = detected
        .and_then(|t| options.iter().position(|candidate| *candidate == t))
        .unwrap_or(options.len() - 1);

    let prompt_msg = match detected {
        Some(t) => format!("Detected {t} project. Select project type:"),
        None => "Select project type:".to_string(),
    };

    Ok(Select::new(&prompt_msg, options)
        .with_starting_cursor(starting_cursor)
        .prompt()?)
}

/// Walk every field of a workspace, seeded with the detected defaults.
fn customize_workspace(
    root: &Path,
    project_type: ProjectType,
    base: Workspace,
) -> Result<Workspace, Box<dyn Error>> {
    println!("\n--- {} ({project_type}) ---", base.dir);

    let dir = prompt_directory(root, &base.dir)?;

    let cmd_default = base.cmd.iter().collect::<Vec<_>>().join(" && ");
    let cmd = Text::new("Build/run command:")
        .with_default(&cmd_default)
        .with_help_message("Runs inside the watch directory — no `cd` prefix needed")
        .with_validator(|input: &str| {
            Ok(if input.trim().is_empty() {
                Validation::Invalid("A command is required".into())
            } else {
                Validation::Valid
            })
        })
        .prompt()?;

    let bin_default = base.bin_path.clone().unwrap_or_default();
    let bin_path = Text::new("Binary path (leave empty for none):")
        .with_default(&bin_default)
        .with_help_message(
            "Compiled binary to restart after the command, relative to the watch directory",
        )
        .prompt()?;
    let bin_path = non_empty(bin_path);

    let bin_arg = match &bin_path {
        Some(_) => {
            let default = base.bin_arg.clone().unwrap_or_default().join(" ");
            let input = Text::new("Binary arguments (space-separated):")
                .with_default(&default)
                .with_help_message("Passed to the binary, e.g. `server --port 8080`")
                .prompt()?;
            parse_args(&input)
        }
        None => None,
    };

    let ignore_default = base.ignore.clone().unwrap_or_default().join(", ");
    let ignore_input = Text::new("Ignore patterns (comma-separated):")
        .with_default(&ignore_default)
        .with_help_message("Globs, directories or names — e.g. target/, node_modules/, *.log")
        .with_validator(|input: &str| {
            Ok(match parse_patterns(input) {
                Some(patterns) => match CompiledFilter::new(&patterns) {
                    Ok(_) => Validation::Valid,
                    Err(e) => Validation::Invalid(e.to_string().into()),
                },
                None => Validation::Valid,
            })
        })
        .prompt()?;

    let env_default = base.env_file.clone().unwrap_or_default();
    let env_file = Text::new("Env file (leave empty for none):")
        .with_default(&env_default)
        .with_help_message("Relative to the watch directory; a leading `/` means the project root")
        .prompt()?;

    let mut workspace = Workspace::new(dir).cmd(cmd);

    if let Some(patterns) = parse_patterns(&ignore_input) {
        workspace = workspace.ignore(patterns);
    }
    if let Some(bin_path) = bin_path {
        workspace = workspace.bin_path(bin_path);
    }
    if let Some(bin_arg) = bin_arg {
        workspace = workspace.bin_arg(bin_arg);
    }
    if let Some(env_file) = non_empty(env_file) {
        workspace = workspace.env_file(env_file);
    }

    Ok(workspace)
}

/// Prompt to add another workspace
fn prompt_add_another() -> Result<bool, Box<dyn Error>> {
    let add_another = Confirm::new("Add another workspace?")
        .with_default(false)
        .with_help_message("Configure additional directories to watch")
        .prompt()?;

    Ok(add_another)
}

/// Prompt for a directory that must exist, relative to the scan root.
fn prompt_directory(root: &Path, default: &str) -> Result<String, Box<dyn Error>> {
    let root = root.to_path_buf();

    let dir = Text::new("Watch directory:")
        .with_default(default)
        .with_help_message("Directory to watch; also the working directory for the command")
        .with_validator(move |input: &str| {
            let input = input.trim();
            Ok(if input.is_empty() {
                Validation::Invalid("A directory is required".into())
            } else if !root.join(input).is_dir() {
                Validation::Invalid(format!("`{input}` is not an existing directory").into())
            } else {
                Validation::Valid
            })
        })
        .prompt()?;

    Ok(dir.trim().to_string())
}

/// `None` for blank input, so optional fields stay unset instead of empty.
fn non_empty(input: String) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Splits a comma-separated pattern list, dropping blanks.
fn parse_patterns(input: &str) -> Option<Vec<String>> {
    let patterns: Vec<String> = input
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if patterns.is_empty() {
        None
    } else {
        Some(patterns)
    }
}

/// Splits whitespace-separated binary arguments, dropping blanks.
fn parse_args(input: &str) -> Option<Vec<String>> {
    let args: Vec<String> = input.split_whitespace().map(|s| s.to_string()).collect();

    if args.is_empty() {
        None
    } else {
        Some(args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_config_dir_for() {
        assert_eq!(config_dir_for(Path::new("")), ".");
        assert_eq!(config_dir_for(Path::new("services/api")), "services/api");
    }

    #[test]
    fn test_workspace_from_detected_uses_relative_dir() {
        let root = tempdir().unwrap();
        fs::create_dir_all(root.path().join("services/api")).unwrap();
        fs::write(
            root.path().join("services/api/go.mod"),
            "module example.com/api",
        )
        .unwrap();

        let detected = detect::scan_projects(root.path(), 2);
        let workspace = workspace_from_detected(root.path(), &detected[0]);

        assert_eq!(workspace.dir, "services/api");
        assert_eq!(workspace.bin_path.as_deref(), Some("./api"));
        assert!(workspace.validate().is_ok());
    }

    #[test]
    fn test_default_workspaces_covers_every_detected_project() {
        let root = tempdir().unwrap();
        fs::create_dir_all(root.path().join("web")).unwrap();
        fs::write(root.path().join("Cargo.toml"), "[package]\nname = \"api\"").unwrap();
        fs::write(
            root.path().join("web/package.json"),
            r#"{"name":"web","scripts":{"dev":"vite"}}"#,
        )
        .unwrap();

        let detected = detect::scan_projects(root.path(), 2);
        let workspaces = default_workspaces(root.path(), &detected);

        assert_eq!(workspaces.len(), 2);

        assert_eq!(workspaces[0].dir, ".");
        assert_eq!(
            workspaces[0].bin_path.as_deref(),
            Some("./target/debug/api")
        );

        assert_eq!(workspaces[1].dir, "web");
        assert_eq!(
            workspaces[1].cmd.iter().collect::<Vec<_>>(),
            vec!["npm run dev"]
        );

        assert!(Config::new(workspaces).validate().is_ok());
    }

    #[test]
    fn test_default_workspaces_falls_back_to_placeholder() {
        let root = tempdir().unwrap();
        let workspaces = default_workspaces(root.path(), &[]);

        assert_eq!(workspaces.len(), 1);
        assert_eq!(workspaces[0].dir, ".");
        assert!(Config::new(workspaces).validate().is_ok());
    }

    #[test]
    fn test_confirm_output_path_allows_missing_file() {
        let dir = tempdir().unwrap();
        let args = InitArgs {
            output: dir.path().join("rustywatch.yaml").display().to_string(),
            ..InitArgs::default()
        };

        assert!(confirm_output_path(&args, false).unwrap());
    }

    #[test]
    fn test_confirm_output_path_errors_for_yes_without_force() {
        let dir = tempdir().unwrap();
        let output = dir.path().join("rustywatch.yaml");
        fs::write(&output, "workspaces: []").unwrap();

        let args = InitArgs {
            output: output.display().to_string(),
            yes: true,
            ..InitArgs::default()
        };

        let err = confirm_output_path(&args, false).unwrap_err();
        assert!(err.to_string().contains("pass --force to overwrite"));
    }

    #[test]
    fn test_confirm_output_path_allows_force_and_dry_run() {
        let dir = tempdir().unwrap();
        let output = dir.path().join("rustywatch.yaml");
        fs::write(&output, "workspaces: []").unwrap();

        let forced = InitArgs {
            output: output.display().to_string(),
            yes: true,
            force: true,
            ..InitArgs::default()
        };
        assert!(confirm_output_path(&forced, false).unwrap());

        let dry_run = InitArgs {
            output: output.display().to_string(),
            yes: true,
            dry_run: true,
            ..InitArgs::default()
        };
        assert!(confirm_output_path(&dry_run, false).unwrap());
    }

    #[test]
    fn test_interactive_init_rejects_missing_directory() {
        let args = InitArgs {
            dir: "/nonexistent/rustywatch/scan-root".to_string(),
            yes: true,
            ..InitArgs::default()
        };

        let err = interactive_init(args).unwrap_err();
        assert!(err.to_string().contains("is not a directory"));
    }

    #[test]
    fn test_parse_patterns() {
        assert_eq!(
            parse_patterns("target/, .git/ ,, node_modules/"),
            Some(vec![
                "target/".to_string(),
                ".git/".to_string(),
                "node_modules/".to_string(),
            ])
        );
        assert_eq!(parse_patterns("   "), None);
        assert_eq!(parse_patterns(""), None);
    }

    #[test]
    fn test_parse_args() {
        assert_eq!(
            parse_args("  server --port  8080 "),
            Some(vec![
                "server".to_string(),
                "--port".to_string(),
                "8080".to_string(),
            ])
        );
        assert_eq!(parse_args("  "), None);
    }

    #[test]
    fn test_non_empty() {
        assert_eq!(
            non_empty("  ./bin/app ".to_string()),
            Some("./bin/app".to_string())
        );
        assert_eq!(non_empty("   ".to_string()), None);
    }
}
