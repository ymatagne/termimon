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
    pub comm: String,
}

/// Get the process table via `ps`.
pub fn list_processes() -> Result<Vec<ProcessInfo>> {
    let output = std::process::Command::new("ps")
        .args(["-eo", "pid,ppid,comm"])
        .output()
        .context("Failed to run ps")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut procs = Vec::new();

    for line in stdout.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let pid = parts[0].parse::<u32>().unwrap_or(0);
            let ppid = parts[1].parse::<u32>().unwrap_or(0);
            let comm_full = parts[2..].join(" ");
            let comm = comm_full
                .rsplit('/')
                .next()
                .unwrap_or(&comm_full)
                .to_string();
            if pid > 0 {
                procs.push(ProcessInfo { pid, ppid, comm });
            }
        }
    }
    Ok(procs)
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
