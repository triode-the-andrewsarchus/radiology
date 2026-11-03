//! Settings-pinned invocation of the shared seL4 configuration-union runner.
use super::{
    command::{CommandPaths, IndexCommand},
    personality::{pipeline_resources, Personality},
    service::GenerationRequest,
    workspace::destination_path,
};
use std::path::PathBuf;

pub(super) fn command(
    request: &GenerationRequest,
    paths: CommandPaths<'_>,
) -> Result<IndexCommand, String> {
    let settings = request
        .settings
        .as_deref()
        .filter(|settings| settings.personality() == Personality::Sel4)
        .ok_or("seL4 generation requires captured seL4 personality settings")?;
    let tools = destination_path(
        request
            .tools
            .managed
            .as_deref()
            .ok_or("seL4 Python provisioning requires a private tools directory")?,
    )?;
    for root in [
        request.root.as_path(),
        request.attached_root.as_path(),
        paths.workspace,
    ] {
        if tools.starts_with(root.canonicalize().map_err(|error| error.to_string())?) {
            return Err("seL4 tools must be outside viewed and isolated source inputs".into());
        }
    }
    let python = settings
        .bootstrap_python()
        .iter()
        .find_map(|name| request.tools.locate_program(name))
        .ok_or("seL4 indexing requires a configured bootstrap Python 3.11+ executable")?;
    let configuration = paths.run.join("personality.toml");
    let script = paths.run.join("sel4_pipeline.py");
    let mut resources: Vec<_> = pipeline_resources()
        .iter()
        .map(|(name, bytes)| (paths.run.join(name), bytes.to_vec()))
        .collect();
    resources.push((configuration.clone(), settings.bytes().to_vec()));
    let mut arguments = vec!["-u".into(), script.into_os_string()];
    for (flag, value) in [
        ("--repository", paths.workspace.to_path_buf()),
        ("--configuration", configuration),
        ("--indexer", paths.executable.to_path_buf()),
        ("--output", paths.output.to_path_buf()),
        ("--artifacts", paths.run.join("sel4-build")),
        ("--tools-directory", tools),
    ] {
        arguments.extend([flag.into(), value.into_os_string()]);
    }
    let mut environment = request.tools.child_environment();
    environment.insert("PYTHONDONTWRITEBYTECODE".into(), "1".into());
    environment.insert("PYTHONUNBUFFERED".into(), "1".into());
    environment.insert("TMPDIR".into(), paths.run.as_os_str().to_owned());
    Ok(IndexCommand {
        executable: python,
        arguments,
        directory: PathBuf::from(paths.workspace),
        environment,
        resources,
    })
}
