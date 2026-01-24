// Author : Eshan Roy <eshanized@proton.me>
// SPDX-License-Identifier: MIT

use rand::Rng;
use std::collections::HashSet;
use std::sync::LazyLock;

const ADJECTIVES: &[&str] = &[
    "brave", "calm", "dark", "eager", "fair", "glad", "happy", "keen", "light", "mild", "neat",
    "pale", "quick", "rare", "safe", "tall", "vast", "warm", "bold", "cool", "deep", "fast",
    "gold", "kind", "live", "pure", "rich", "slim", "soft", "wise",
];

const NOUNS: &[&str] = &[
    "bear", "crow", "deer", "dove", "eagle", "fawn", "goat", "hawk", "ibis", "jade", "kite",
    "lark", "moth", "newt", "orca", "puma", "quail", "reef", "seal", "tern", "vole", "wolf",
    "wren", "yak", "bass", "crab", "duck", "elm", "frog", "gull",
];

static ADJECTIVE_SET: LazyLock<HashSet<&'static str>> =
    LazyLock::new(|| ADJECTIVES.iter().copied().collect());

static NOUN_SET: LazyLock<HashSet<&'static str>> =
    LazyLock::new(|| NOUNS.iter().copied().collect());

/// Builds a subdomain like "brave-eagle-a1b2c3d4" by picking a random
/// adjective and noun from our word lists, then appending 8 hex characters
/// for uniqueness. The hex part comes from random bytes so collisions are
/// extremely unlikely even at scale.
pub fn generate() -> String {
    let mut rng = rand::thread_rng();

    let adj = ADJECTIVES[rng.gen_range(0..ADJECTIVES.len())];
    let noun = NOUNS[rng.gen_range(0..NOUNS.len())];

    let mut hex_bytes = [0u8; 4];
    rng.fill(&mut hex_bytes);
    let hex_suffix = hex::encode(hex_bytes);

    format!("{adj}-{noun}-{hex_suffix}")
}

/// Checks whether a string matches the expected subdomain pattern:
/// exactly three parts separated by hyphens, where the first part is
/// a known adjective, the second is a known noun, and the third is
/// an 8-character lowercase hex string. This prevents subdomain injection
/// and ensures only our generated (or validated custom) names are used.
pub fn is_valid(subdomain: &str) -> bool {
    let parts: Vec<&str> = subdomain.split('-').collect();
    if parts.len() != 3 {
        return false;
    }

    let adj_ok = ADJECTIVE_SET.contains(parts[0]);
    let noun_ok = NOUN_SET.contains(parts[1]);
    let hex_ok = parts[2].len() == 8
        && parts[2]
            .chars()
            .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c));

    adj_ok && noun_ok && hex_ok
}

/// Validates a custom subdomain that users can reserve. Custom names
/// must be between 3 and 30 characters, start with a letter, and only
/// contain lowercase alphanumeric characters or hyphens. No leading,
/// trailing, or consecutive hyphens allowed.
pub fn is_valid_custom(subdomain: &str) -> bool {
    if subdomain.len() < 3 || subdomain.len() > 30 {
        return false;
    }

    if !subdomain.starts_with(|c: char| c.is_ascii_lowercase()) {
        return false;
    }

    if subdomain.starts_with('-') || subdomain.ends_with('-') {
        return false;
    }

    if subdomain.contains("--") {
        return false;
    }

    subdomain
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_subdomains_are_valid() {
        for _ in 0..100 {
            let sub = generate();
            assert!(is_valid(&sub), "generated subdomain should be valid: {sub}");
        }
    }

    #[test]
    fn generated_subdomains_are_unique() {
        let subs: HashSet<String> = (0..50).map(|_| generate()).collect();
        assert_eq!(
            subs.len(),
            50,
            "50 generated subdomains should all be unique"
        );
    }

    #[test]
    fn rejects_invalid_formats() {
        assert!(!is_valid(""));
        assert!(!is_valid("only-two"));
        assert!(!is_valid("too-many-parts-here"));
        assert!(!is_valid("unknown-eagle-abcdef01"));
        assert!(!is_valid("brave-unknown-abcdef01"));
        assert!(!is_valid("brave-eagle-short"));
        assert!(!is_valid("brave-eagle-ABCDEF01"));
    }

    #[test]
    fn custom_subdomain_validation() {
        assert!(is_valid_custom("myapp"));
        assert!(is_valid_custom("my-cool-app"));
        assert!(is_valid_custom("app123"));

        assert!(!is_valid_custom("ab"));
        assert!(!is_valid_custom("-start"));
        assert!(!is_valid_custom("end-"));
        assert!(!is_valid_custom("bad--double"));
        assert!(!is_valid_custom("1starts-with-digit"));
        assert!(!is_valid_custom("has spaces"));
        assert!(!is_valid_custom("HAS_CAPS"));
    }
}
