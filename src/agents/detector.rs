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
    // Strategy 1: lsof for processes that have a real cwd
    if let Some(dir) = get_cwd_via_lsof(pid) {
        if dir != "/" {
            return Some(dir);
        }
    }

    // Strategy 2: Check parent process cwd (Claude Code is spawned from a shell)
    if let Some(ppid) = get_ppid(pid) {
        if let Some(dir) = get_cwd_via_lsof(ppid) {
            if dir != "/" {
                return Some(dir);
            }
        }
        // Strategy 3: Check grandparent (shell → tmux → zsh)
        if let Some(gppid) = get_ppid(ppid) {
            if let Some(dir) = get_cwd_via_lsof(gppid) {
                if dir != "/" {
                    return Some(dir);
                }
            }
        }
    }

    None
}

fn get_cwd_via_lsof(pid: u32) -> Option<String> {
    let output = std::process::Command::new("lsof")
        .args(["-d", "cwd", "-p", &pid.to_string(), "-Fn"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if let Some(path) = line.strip_prefix('n') {
            if path.starts_with('/') {
                return Some(path.to_string());
            }
        }
    }
    None
}

fn get_ppid(pid: u32) -> Option<u32> {
    let output = std::process::Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "ppid="])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output()
        .ok()?;
    String::from_utf8_lossy(&output.stdout).trim().parse().ok()
}

/// Find active Claude project directories by looking at recently modified JSONL files.
/// Returns project dirs sorted by most recently modified (most active first).
/// Each project dir is the real filesystem path decoded from the encoded dirname.
pub fn find_active_claude_projects(max_age_secs: u64) -> Vec<(String, String)> {
    // Returns Vec<(encoded_project_dir, real_project_path)>
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return Vec::new(),
    };
    let projects_dir = home.join(".claude").join("projects");
    if !projects_dir.exists() {
        return Vec::new();
    }

    let now = std::time::SystemTime::now();
    let mut project_times: HashMap<String, (std::time::SystemTime, String)> = HashMap::new();

    // Walk all JSONL files, find most recent per project dir
    if let Ok(entries) = std::fs::read_dir(&projects_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() { continue; }
            let encoded_name = path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            
            // Find most recently modified JSONL in this project dir (and subdirs)
            let mut newest: Option<std::time::SystemTime> = None;
            visit_jsonl_files(&path, &mut |f: &std::path::Path| {
                if let Ok(meta) = f.metadata() {
                    if let Ok(mtime) = meta.modified() {
                        if newest.map_or(true, |n| mtime > n) {
                            newest = Some(mtime);
                        }
                    }
                }
            });

            if let Some(mtime) = newest {
                if let Ok(age) = now.duration_since(mtime) {
                    if age.as_secs() <= max_age_secs {
                        let real_path = decode_project_dir(&encoded_name);
                        project_times.insert(encoded_name.clone(), (mtime, real_path));
                    }
                }
            }
        }
    }

    // Sort by most recent first
    let mut results: Vec<_> = project_times.into_iter()
        .map(|(encoded, (_, real))| (encoded, real))
        .collect();
    results.sort();
    results
}

fn visit_jsonl_files(dir: &std::path::Path, cb: &mut dyn FnMut(&std::path::Path)) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                visit_jsonl_files(&p, cb);
            } else if p.extension().and_then(|e| e.to_str()) == Some("jsonl") {
                cb(&p);
            }
        }
    }
}

/// Decode an encoded project directory name back to a real path.
/// "-Users-yan-github-com-foo" → "/Users/yan/github.com/foo"  
/// "-private-tmp-termimon" → "/private/tmp/termimon"
fn decode_project_dir(encoded: &str) -> String {
    // The encoding replaces "/" with "-" and strips the leading "/"
    // But we can't perfectly reverse it. Use history.jsonl for real paths.
    let home = dirs::home_dir().unwrap_or_default();
    let history_path = home.join(".claude").join("history.jsonl");
    
    // Build a map of encoded → real from history.jsonl
    if let Ok(content) = std::fs::read_to_string(&history_path) {
        for line in content.lines().rev() {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
                if let Some(project) = v.get("project").and_then(|p| p.as_str()) {
                    let test_encoded = crate::agents::cost::encode_working_dir(project);
                    if test_encoded == encoded {
                        return project.to_string();
                    }
                }
            }
        }
    }
    
    // Fallback: best-effort decode (replace leading - with /)
    format!("/{}", encoded.replace('-', "/"))
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
