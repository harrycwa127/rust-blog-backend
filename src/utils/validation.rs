use regex::Regex;
use once_cell::sync::Lazy;
use std::sync::LazyLock;

pub static TAG_NAME_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[\p{L}\p{N}_-]+$").expect("TAG_NAME_REGEX"));

pub static COLOR_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^#[0-9A-Fa-f]{6}$").expect("COLOR_REGEX"));

pub static SLUG_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-z0-9-]+$").expect("SLUG_REGEX"));

pub fn generate_random_color() -> String {
    use rand::Rng;

    let colors = [
        "#EF4444", "#F97316", "#F59E0B", "#EAB308", "#84CC16",
        "#22C55E", "#10B981", "#14B8A6", "#06B6D4", "#0EA5E9",
        "#3B82F6", "#6366F1", "#8B5CF6", "#A855F7", "#D946EF",
        "#EC4899", "#F43F5E", "#64748B", "#6B7280", "#374151",
    ];

    let mut rng = rand::thread_rng();
    colors[rng.gen_range(0..colors.len())].to_string()
}

pub fn sanitize_tag_name(name: &str) -> String {
    name.trim()
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else if c.is_whitespace() {
                '-'
            } else {
                '_'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}