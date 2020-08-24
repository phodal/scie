extern crate onig;

pub mod grammar;
pub mod grammar_registry;
pub mod inter;
pub mod rule;
pub mod reg_exp_source;

#[cfg(test)]
mod tests {
    use onig::*;

    #[test]
    fn should_run_for_onig() {
        let regex = Regex::new("e(l+)").unwrap();
        for (i, pos) in regex.captures("hello").unwrap().iter_pos().enumerate() {
            match pos {
                Some((beg, end)) => println!("Group {} captured in position {}:{}", i, beg, end),
                None => println!("Group {} is not captured", i),
            }
        }
    }
}
