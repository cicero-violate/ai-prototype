//! Skill registry — deterministic capability selection by objective and strategy.
//!
//! Adapted from canon-skills/src/lib.rs. Skill markdown files use YAML frontmatter
//! to declare which objectives and strategies they support. The registry caches
//! parsed skills in a read-write lock and supports inclusion chains.
//!
//! Skill files are read-only inputs; any update to skill routing must flow through
//! a receipt-gated command, not by writing back to the registry at runtime.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock, RwLock};

use serde::Deserialize;

// ---------------------------------------------------------------------------
// Objective and strategy enums — stable identity for skill matching
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DevelopmentObjectiveKind {
    ReduceCompilerFailures,
    ReduceContradictionRate,
    IncreaseTestCoverage,
    DecreaseInvalidPlanRate,
    ReduceStalledLoopFrequency,
    ImproveModuleCohesion,
}

impl DevelopmentObjectiveKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ReduceCompilerFailures => "reduce_compiler_failures",
            Self::ReduceContradictionRate => "reduce_contradiction_rate",
            Self::IncreaseTestCoverage => "increase_test_coverage",
            Self::DecreaseInvalidPlanRate => "decrease_invalid_plan_rate",
            Self::ReduceStalledLoopFrequency => "reduce_stalled_loop_frequency",
            Self::ImproveModuleCohesion => "improve_module_cohesion",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DevelopmentStrategyKind {
    FixConfigLintPolicy,
    ApplyTargetedCompilerRepair,
    DiscoverTestSurface,
    AddRegressionTest,
    SimplifyPlanBatch,
    RealignObjectiveFlow,
    RefreshContextBeforeRetry,
    CreateMissingModules,
    RestructureModules,
}

impl DevelopmentStrategyKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FixConfigLintPolicy => "fix_config_lint_policy",
            Self::ApplyTargetedCompilerRepair => "apply_targeted_compiler_repair",
            Self::DiscoverTestSurface => "discover_test_surface",
            Self::AddRegressionTest => "add_regression_test",
            Self::SimplifyPlanBatch => "simplify_plan_batch",
            Self::RealignObjectiveFlow => "realign_objective_flow",
            Self::RefreshContextBeforeRetry => "refresh_context_before_retry",
            Self::CreateMissingModules => "create_missing_modules",
            Self::RestructureModules => "restructure_modules",
        }
    }
}

// ---------------------------------------------------------------------------
// LLM effort hint
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LlmEffort {
    Low,
    Medium,
    High,
}

// ---------------------------------------------------------------------------
// Skill
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub effort: Option<LlmEffort>,
    pub objectives: Vec<DevelopmentObjectiveKind>,
    pub strategies: Vec<DevelopmentStrategyKind>,
    pub tools: Vec<String>,
    pub preconditions: Vec<String>,
    pub prompt: String,
}

impl Skill {
    /// True if this skill applies to the given objective/strategy.
    /// A `None` filter matches all skills. An empty `objectives` or `strategies`
    /// list in the skill file means "applies to everything".
    pub fn supports(
        &self,
        objective: Option<DevelopmentObjectiveKind>,
        strategy: Option<DevelopmentStrategyKind>,
    ) -> bool {
        let obj_ok = objective
            .map(|o| self.objectives.is_empty() || self.objectives.contains(&o))
            .unwrap_or(true);
        let str_ok = strategy
            .map(|s| self.strategies.is_empty() || self.strategies.contains(&s))
            .unwrap_or(true);
        obj_ok && str_ok
    }
}

// ---------------------------------------------------------------------------
// Frontmatter schema
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Default)]
struct SkillFrontmatter {
    name: Option<String>,
    description: Option<String>,
    effort: Option<LlmEffort>,
    includes: Option<Vec<String>>,
    objectives: Option<Vec<DevelopmentObjectiveKind>>,
    strategies: Option<Vec<DevelopmentStrategyKind>>,
    tools: Option<Vec<String>>,
    preconditions: Option<Vec<String>>,
}

// ---------------------------------------------------------------------------
// Registry
// ---------------------------------------------------------------------------

pub struct SkillRegistry {
    skills_dir: PathBuf,
    cache: RwLock<HashMap<String, Arc<Skill>>>,
}

impl SkillRegistry {
    pub fn new(skills_dir: PathBuf) -> Self {
        Self {
            skills_dir,
            cache: RwLock::new(HashMap::new()),
        }
    }

    /// Load a skill by its path relative to the skills directory (no `.md` extension).
    pub fn load(&self, skill_path: &str) -> Result<Arc<Skill>, String> {
        self.load_inner(skill_path, &mut HashSet::new())
    }

    /// Evict a single skill from cache (force reload on next access).
    pub fn invalidate(&self, skill_path: &str) {
        if let Ok(mut w) = self.cache.write() {
            w.remove(skill_path);
        }
    }

    /// Evict all cached skills.
    pub fn invalidate_all(&self) {
        if let Ok(mut w) = self.cache.write() {
            w.clear();
        }
    }

    fn load_inner(
        &self,
        skill_path: &str,
        seen: &mut HashSet<String>,
    ) -> Result<Arc<Skill>, String> {
        {
            if let Ok(r) = self.cache.read() {
                if let Some(cached) = r.get(skill_path) {
                    return Ok(cached.clone());
                }
            }
        }
        if !seen.insert(skill_path.to_string()) {
            return Err(format!("cycle detected while loading skill {skill_path}"));
        }
        let path = self.skills_dir.join(format!("{skill_path}.md"));
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("failed to read skill {skill_path}: {e}"))?;
        let (front, body) = split_frontmatter(&content);
        let fm: SkillFrontmatter = if let Some(raw) = front {
            serde_yaml::from_str(raw)
                .map_err(|e| format!("failed to parse frontmatter for {skill_path}: {e}"))?
        } else {
            SkillFrontmatter::default()
        };
        let mut included_prompts: Vec<String> = Vec::new();
        if let Some(includes) = &fm.includes {
            for inc in includes {
                let child = self.load_inner(inc, seen)?;
                included_prompts.push(child.prompt.clone());
            }
        }
        seen.remove(skill_path);
        let mut prompt_parts = included_prompts;
        prompt_parts.push(body.trim().to_string());
        let skill = Arc::new(Skill {
            name: fm.name.unwrap_or_else(|| skill_path.to_string()),
            description: fm.description.unwrap_or_default(),
            effort: fm.effort,
            objectives: fm.objectives.unwrap_or_default(),
            strategies: fm.strategies.unwrap_or_default(),
            tools: fm.tools.unwrap_or_default(),
            preconditions: fm.preconditions.unwrap_or_default(),
            prompt: prompt_parts.join("\n\n"),
        });
        if let Ok(mut w) = self.cache.write() {
            w.insert(skill_path.to_string(), skill.clone());
        }
        Ok(skill)
    }

    /// Return all skills matching both objective and strategy, sorted by path.
    pub fn select_for(
        &self,
        objective: DevelopmentObjectiveKind,
        strategy: DevelopmentStrategyKind,
    ) -> Result<Vec<Arc<Skill>>, String> {
        self.select_for_scope("", objective, strategy)
    }

    /// Like `select_for` but restricted to skills under `scope/`.
    pub fn select_for_scope(
        &self,
        scope: &str,
        objective: DevelopmentObjectiveKind,
        strategy: DevelopmentStrategyKind,
    ) -> Result<Vec<Arc<Skill>>, String> {
        let mut out = Vec::new();
        for skill_path in collect_skill_paths(&self.skills_dir, scope)? {
            let skill = self.load(&skill_path)?;
            if skill.supports(Some(objective), Some(strategy)) {
                out.push(skill);
            }
        }
        Ok(out)
    }
}

fn collect_skill_paths(root: &Path, scope: &str) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    let normalized_scope = scope.trim_matches('/');
    collect_skill_paths_inner(root, root, normalized_scope, &mut out)
        .map_err(|e| format!("failed to collect skill paths: {e}"))?;
    out.sort();
    Ok(out)
}

fn collect_skill_paths_inner(
    root: &Path,
    current: &Path,
    scope: &str,
    out: &mut Vec<String>,
) -> std::io::Result<()> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_skill_paths_inner(root, &path, scope, out)?;
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let Ok(rel) = path.strip_prefix(root) else {
            continue;
        };
        let mut skill_path = rel.to_string_lossy().replace('\\', "/");
        if let Some(stripped) = skill_path.strip_suffix(".md") {
            skill_path = stripped.to_string();
        }
        if !scope.is_empty() && !skill_path.starts_with(&format!("{scope}/")) {
            continue;
        }
        out.push(skill_path);
    }
    Ok(())
}

fn split_frontmatter(content: &str) -> (Option<&str>, &str) {
    let trimmed = content.trim_start();
    if let Some(rest) = trimmed.strip_prefix("---") {
        if let Some(end) = rest.find("\n---") {
            let fm = &rest[..end];
            let body = &rest[end + 4..];
            return (Some(fm), body);
        }
    }
    (None, content)
}

/// Process-global skill registry backed by the `CANON_SKILLS_DIR` environment variable.
/// Returns `None` when the env var is not set.
pub fn global_registry() -> Option<&'static SkillRegistry> {
    static REG: OnceLock<Option<SkillRegistry>> = OnceLock::new();
    REG.get_or_init(|| {
        let dir = std::env::var("CANON_SKILLS_DIR").ok().map(PathBuf::from)?;
        Some(SkillRegistry::new(dir))
    })
    .as_ref()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU32, Ordering};

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    fn test_tmp_dir() -> PathBuf {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../state/tmp");
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn scratch_dir() -> (PathBuf, tempfile::TempDir) {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let tmp = tempfile::Builder::new()
            .prefix(&format!("ai-skills-test-{}-{n}-", std::process::id()))
            .tempdir_in(test_tmp_dir())
            .unwrap();
        let dir = tmp.path().to_owned();
        (dir, tmp)
    }

    fn write_skill(dir: &Path, name: &str, content: &str) {
        fs::write(dir.join(format!("{name}.md")), content).unwrap();
    }

    #[test]
    fn skill_supports_empty_lists_matches_all() {
        let skill = Skill {
            name: "any".to_string(),
            description: String::new(),
            effort: None,
            objectives: Vec::new(),
            strategies: Vec::new(),
            tools: Vec::new(),
            preconditions: Vec::new(),
            prompt: String::new(),
        };
        assert!(skill.supports(
            Some(DevelopmentObjectiveKind::ReduceCompilerFailures),
            Some(DevelopmentStrategyKind::FixConfigLintPolicy)
        ));
        assert!(skill.supports(None, None));
    }

    #[test]
    fn skill_supports_filtered_objective_rejects_mismatch() {
        let skill = Skill {
            name: "targeted".to_string(),
            description: String::new(),
            effort: None,
            objectives: vec![DevelopmentObjectiveKind::ReduceCompilerFailures],
            strategies: Vec::new(),
            tools: Vec::new(),
            preconditions: Vec::new(),
            prompt: String::new(),
        };
        assert!(skill.supports(Some(DevelopmentObjectiveKind::ReduceCompilerFailures), None));
        assert!(!skill.supports(Some(DevelopmentObjectiveKind::IncreaseTestCoverage), None));
    }

    #[test]
    fn registry_loads_skill_from_markdown() {
        let (dir, _tmp) = scratch_dir();
        write_skill(
            &dir,
            "fix-lint",
            r#"---
name: fix-lint
description: Fix lint policy
objectives:
  - reduce_compiler_failures
strategies:
  - fix_config_lint_policy
---
Apply the lint fix here.
"#,
        );
        let reg = SkillRegistry::new(dir.clone());
        let skill = reg.load("fix-lint").expect("should load");
        assert_eq!(skill.name, "fix-lint");
        assert!(skill
            .objectives
            .contains(&DevelopmentObjectiveKind::ReduceCompilerFailures));
        assert!(skill
            .strategies
            .contains(&DevelopmentStrategyKind::FixConfigLintPolicy));
        assert!(skill.prompt.contains("Apply the lint fix here."));
    }

    #[test]
    fn registry_select_for_returns_matching_skills() {
        let (dir, _tmp) = scratch_dir();
        write_skill(
            &dir,
            "a",
            "---\nobjectives:\n  - reduce_compiler_failures\nstrategies:\n  - fix_config_lint_policy\n---\nA skill.\n",
        );
        write_skill(
            &dir,
            "b",
            "---\nobjectives:\n  - increase_test_coverage\nstrategies:\n  - add_regression_test\n---\nB skill.\n",
        );
        let reg = SkillRegistry::new(dir.clone());
        let matched = reg
            .select_for(
                DevelopmentObjectiveKind::ReduceCompilerFailures,
                DevelopmentStrategyKind::FixConfigLintPolicy,
            )
            .unwrap();
        assert_eq!(matched.len(), 1);
        assert_eq!(matched[0].name, "a");
    }

    #[test]
    fn registry_invalidate_forces_reload() {
        let (dir, _tmp) = scratch_dir();
        write_skill(&dir, "s", "---\nname: v1\n---\nVersion 1.\n");
        let reg = SkillRegistry::new(dir.clone());
        let first = reg.load("s").unwrap();
        assert_eq!(first.name, "v1");

        write_skill(&dir, "s", "---\nname: v2\n---\nVersion 2.\n");
        let cached = reg.load("s").unwrap();
        assert_eq!(cached.name, "v1"); // still cached

        reg.invalidate("s");
        let reloaded = reg.load("s").unwrap();
        assert_eq!(reloaded.name, "v2"); // fresh load
    }

    #[test]
    fn registry_detects_include_cycle() {
        let (dir, _tmp) = scratch_dir();
        write_skill(&dir, "a", "---\nincludes:\n  - b\n---\nA body.\n");
        write_skill(&dir, "b", "---\nincludes:\n  - a\n---\nB body.\n");
        let reg = SkillRegistry::new(dir.clone());
        let err = reg.load("a").unwrap_err();
        assert!(err.contains("cycle"), "{err}");
    }

    #[test]
    fn split_frontmatter_parses_correct_sections() {
        let content = "---\nname: foo\n---\nBody here.\n";
        let (fm, body) = split_frontmatter(content);
        // The `\n` before `---` is the delimiter, not part of the FM slice.
        assert_eq!(fm, Some("\nname: foo"));
        assert!(body.contains("Body here."));
    }

    #[test]
    fn split_frontmatter_returns_none_when_no_markers() {
        let content = "Just a body.\n";
        let (fm, body) = split_frontmatter(content);
        assert!(fm.is_none());
        assert!(body.contains("Just a body."));
    }
}
