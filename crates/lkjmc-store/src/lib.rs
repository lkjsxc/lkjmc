#![forbid(unsafe_code)]

pub const COMPONENT: &str = "lkjmc-store";

pub fn component_name() -> &'static str {
    COMPONENT
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_component_name() {
        assert_eq!(component_name(), "lkjmc-store");
    }
}
