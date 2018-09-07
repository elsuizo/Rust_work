/// Given two strings of letters, determine whether the second can be
/// made from the first by removing one letter. The remaining letters
/// must stay in the same order.
/// Examples:
///
/// funnel("leave", "eave") => true
/// funnel("reset", "rest") => true
/// funnel("dragoon", "dragon") => true
/// funnel("eave", "leave") => false
/// funnel("sleet", "lets") => false
/// funnel("skiff", "ski") => false/

pub fn funnel(src: &str, dest: &str) -> bool {
    // A funneled word must always be one less in length.
    if src.len() != dest.len() + 1 {
        return false;
    }
    let mut src = src.chars();
    let mut dest = dest.chars();

    while let (Some(s), Some(d)) = (src.next(), dest.next()) {
        // Find the first mismatched character
        if s != d {
            let s = src.next().unwrap();

            // The next character in src must match this character of dest.
            return s == d

            // .. and the rest of src must match the rest of dest.
                && src.eq(dest);
        }
    }
    // No mismatch found, then the last letter was skipped:
    true
}

fn main() {
    assert_eq!(funnel("reset", "rest"), true);
}
