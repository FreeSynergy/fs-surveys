//! Spelling of names: wire names are camel case, choice keys and label keys are kebab case.
//!
//! Pattern: none (two pure functions). `anonymousOpen` is the name in `surveys.capnp` and in the stored
//! JSON; `anonymous-open` is the key a description may carry (the description rule forbids capitals).

/// `anonymousOpen` -> `anonymous-open`.
#[must_use]
pub fn kebab(name: &str) -> String {
    let mut out = String::new();
    for c in name.chars() {
        if c.is_ascii_uppercase() {
            out.push('-');
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

/// `anonymous-open` -> `anonymousOpen`.
#[must_use]
pub fn camel(key: &str) -> String {
    let mut out = String::new();
    let mut up = false;
    for c in key.chars() {
        if c == '-' {
            up = true;
        } else if up {
            out.push(c.to_ascii_uppercase());
            up = false;
        } else {
            out.push(c);
        }
    }
    out
}
