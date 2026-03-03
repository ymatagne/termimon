//! Process tree inspection and output pattern matching
//!
//! Walks the process tree starting from tmux pane PIDs to find known agent
//! processes, then matches terminal output against agent-specific patterns.

use anyhow::{Context, Result};
use std::collections::HashMap;

/// Info about a single process from `ps`.
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub ppid: u32,
    pub cpu_pct: f32,
    pub mem_mb: f64,
    pub comm: String,
}

/// Get the process table via `ps` (includes CPU % and RSS).
pub fn list_processes() -> Result<Vec<ProcessInfo>> {
    let output = std::process::Command::new("ps")
        .args(["-eo", "pid,ppid,%cpu,rss,comm"])
        .output()
        .context("Failed to run ps")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut procs = Vec::new();

    for line in stdout.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 5 {
            let pid = parts[0].parse::<u32>().unwrap_or(0);
            let ppid = parts[1].parse::<u32>().unwrap_or(0);
            let cpu_pct = parts[2].parse::<f32>().unwrap_or(0.0);
            let rss_kb = parts[3].parse::<f64>().unwrap_or(0.0);
            let mem_mb = rss_kb / 1024.0;
            let comm_full = parts[4..].join(" ");
            let comm = comm_full
                .rsplit('/')
                .next()
                .unwrap_or(&comm_full)
                .to_string();
            if pid > 0 {
                procs.push(ProcessInfo { pid, ppid, cpu_pct, mem_mb, comm });
            }
        }
    }
    Ok(procs)
}

/// Get the working directory of a process via `lsof`.
pub fn get_working_dir(pid: u32) -> Option<String> {
    // Strategy 1: Check process command line args for --project or working dir
    // Claude Code often has the project path in its args
    if let Some(dir) = get_working_dir_from_cmdline(pid) {
        return Some(dir);
    }

    // Strategy 2: lsof (works for most processes, but Claude Code returns "/")
    let output = std::process::Command::new("lsof")
        .args(["-d", "cwd", "-p", &pid.to_string(), "-Fn"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if let Some(path) = line.strip_prefix('n') {
            if path.starts_with('/') && path != "/" {
                return Some(path.to_string());
            }
        }
    }

    // Strategy 3: Check parent process working dir
    // Claude Code CLI is spawned from a shell — the parent shell's cwd is the project dir
    if let Some(dir) = get_parent_working_dir(pid) {
        if dir != "/" {
            return Some(dir);
        }
    }

    None
}

/// Try to get working dir from process command line (macOS: ps -o args)
fn get_working_dir_from_cmdline(pid: u32) -> Option<String> {
    let output = std::process::Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "args="])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output()
        .ok()?;

    let args = String::from_utf8_lossy(&output.stdout).to_string();
    // Look for --project flag or a directory path argument
    let parts: Vec<&str> = args.split_whitespace().collect();
    for (i, part) in parts.iter().enumerate() {
        if *part == "--project" || *part == "-p" {
            if let Some(dir) = parts.get(i + 1) {
                if std::path::Path::new(dir).is_dir() {
                    return Some(dir.to_string());
                }
            }
        }
    }
    None
}

/// Get the parent process's working directory
fn get_parent_working_dir(pid: u32) -> Option<String> {
    // Get parent PID
    let output = std::process::Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "ppid="])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output()
        .ok()?;

    let ppid: u32 = String::from_utf8_lossy(&output.stdout).trim().parse().ok()?;
    if ppid <= 1 {
        return None;
    }

    // Get parent's cwd via lsof
    let output = std::process::Command::new("lsof")
        .args(["-d", "cwd", "-p", &ppid.to_string(), "-Fn"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if let Some(path) = line.strip_prefix('n') {
            if path.starts_with('/') && path != "/" {
                return Some(path.to_string());
            }
        }
    }
    None
}

/// Build a pid → children mapping.
pub fn build_process_tree(procs: &[ProcessInfo]) -> HashMap<u32, Vec<u32>> {
    let mut tree: HashMap<u32, Vec<u32>> = HashMap::new();
    for p in procs {
        tree.entry(p.ppid).or_default().push(p.pid);
    }
    tree
}

/// Get all descendant processes of `root_pid`.
pub fn descendant_processes(root_pid: u32, procs: &[ProcessInfo]) -> Vec<ProcessInfo> {
    let tree = build_process_tree(procs);
    let by_pid: HashMap<u32, &ProcessInfo> = procs.iter().map(|p| (p.pid, p)).collect();

    let mut result = Vec::new();
    let mut stack = vec![root_pid];

    while let Some(pid) = stack.pop() {
        if let Some(info) = by_pid.get(&pid) {
            result.push((*info).clone());
        }
        if let Some(children) = tree.get(&pid) {
            stack.extend(children);
        }
    }
    result
}

/// Look for a process with a matching command name in the descendants of `root_pid`.
pub fn find_process_in_tree(
    root_pid: u32,
    process_names: &[&str],
    procs: &[ProcessInfo],
) -> Option<ProcessInfo> {
    let descendants = descendant_processes(root_pid, procs);
    for desc in &descendants {
        let comm_lower = desc.comm.to_lowercase();
        for name in process_names {
            if comm_lower.contains(&name.to_lowercase()) {
                return Some(desc.clone());
            }
        }
    }
    None
}

/// Check if a process is still alive.
pub fn is_process_alive(pid: u32) -> bool {
    unsafe { libc::kill(pid as i32, 0) == 0 }
}

/// Check if the last non-empty line matches any of the prompt patterns.
pub fn is_at_prompt(content: &str, prompt_patterns: &[&str]) -> bool {
    let last_line = content.lines().rev().find(|l| !l.trim().is_empty());
    match last_line {
        Some(line) => {
            let trimmed = line.trim();
            prompt_patterns.iter().any(|p| trimmed.contains(p))
        }
        None => false,
    }
}

/// Search pane content (from bottom up) for any of the given patterns.
pub fn match_output_patterns(content: &str, patterns: &[&str]) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    for line in lines.iter().rev() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        for pattern in patterns {
            if trimmed.contains(pattern) {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}
