use std::collections::HashSet;
use std::ops::RangeInclusive;

use lazy_static::lazy_static;
use regex::{Captures, Regex};

fn main() {
    let mut lines = include_str!("../input.txt").lines();
    let rules = read_rules(&mut lines);

    let my_passport = read_passports(&mut lines).first().unwrap().clone();
    let near_passports = read_passports(&mut lines);
    let invalid_fields = find_all_invalid_fields(&near_passports, &rules);
    println!(
        "answer 1 is {}",
        invalid_fields.iter().map(|field| **field).sum::<i64>()
    );

    let valid_passports = filter_invalid(near_passports, &rules);
    let valid_positions = find_all_valid_positions(&rules, &valid_passports);
    let determined_positions = determine_field_positions(valid_positions);
    let part_2: i64 = determined_positions
        .iter()
        .filter(|(_, name)| name.starts_with("departure"))
        .map(|(field, _)| my_passport[*field])
        .product();

    println!("part 2: {}", part_2);
}

#[derive(Debug)]
struct Rule {
    name: String,
    range1: RangeInclusive<i64>,
    range2: RangeInclusive<i64>,
}

impl Rule {
    fn new(name: String, range1: RangeInclusive<i64>, range2: RangeInclusive<i64>) -> Rule {
        Rule {
            name,
            range1,
            range2,
        }
    }

    fn valid(&self, field: &i64) -> bool {
        self.range1.contains(field) || self.range2.contains(field)
    }
}

lazy_static! {
    static ref RULE_REGEX: Regex = Regex::new(r"(?P<name>[\w\s]+): (?P<range_1_low>\d+)-(?P<range_1_high>\d+) or (?P<range_2_low>\d+)-(?P<range_2_high>\d+)$").unwrap();
    static ref FIELD_REGEX: Regex = Regex::new(r"(?P<value>\d+),?").unwrap();
}

fn read_rules<'a>(lines: &mut impl Iterator<Item = &'a str>) -> Vec<Rule> {
    let mut rules: Vec<Rule> = Vec::new();
    for line in lines {
        if let Some(rule_caps) = RULE_REGEX.captures(line) {
            let range_1_low = rule_caps["range_1_low"].parse::<i64>().unwrap();
            let range_1_high = rule_caps["range_1_high"].parse::<i64>().unwrap();
            let range_2_low: i64 = rule_caps["range_2_low"].parse::<i64>().unwrap();
            let range_2_high: i64 = rule_caps["range_2_high"].parse::<i64>().unwrap();
            rules.push(Rule::new(
                rule_caps["name"].to_string(),
                range_1_low..=range_1_high,
                range_2_low..=range_2_high,
            ))
        } else {
            break;
        }
    }
    rules
}

fn read_passports<'a>(lines: &mut impl Iterator<Item = &'a str>) -> Vec<Vec<i64>> {
    let mut passports = Vec::new();
    lines.next();
    for line in lines {
        let caps: Vec<Captures> = FIELD_REGEX.captures_iter(line).collect();
        if !caps.is_empty() {
            let mut passport = Vec::new();
            for field in caps.into_iter() {
                passport.push(field["value"].parse::<i64>().unwrap());
            }
            passports.push(passport)
        } else {
            break;
        }
    }
    passports
}

fn find_invalid_fields<'a>(passport: &'a [i64], rules: &[Rule]) -> Vec<&'a i64> {
    let fields = passport
        .iter()
        .filter(|field| rules.iter().all(|rule| !rule.valid(*field)))
        .collect();
    fields
}

fn find_all_invalid_fields<'a>(passports: &'a [Vec<i64>], rules: &[Rule]) -> Vec<&'a i64> {
    passports
        .iter()
        .map(|passport| find_invalid_fields(&passport, rules))
        .filter(|invalid_fields| !invalid_fields.is_empty())
        .flatten()
        .collect()
}

fn filter_invalid(passports: Vec<Vec<i64>>, rules: &[Rule]) -> Vec<Vec<i64>> {
    passports
        .into_iter()
        .filter(|passport| find_invalid_fields(passport, rules).is_empty())
        .collect()
}

fn is_valid_in_position(rule: &Rule, position: usize, passports: &[Vec<i64>]) -> bool {
    passports
        .iter()
        .all(|passport| rule.valid(&passport[position]))
}

fn find_all_valid_positions(rules: &[Rule], passports: &[Vec<i64>]) -> Vec<Vec<(usize, String)>> {
    let mut positions = Vec::new();
    for rule in rules {
        let mut rule_positions = Vec::new();
        for position in 0..rules.len() {
            if is_valid_in_position(rule, position, passports) {
                rule_positions.push((position, rule.name.clone()));
            }
        }
        positions.push(rule_positions);
    }
    positions
}

fn determine_field_positions(mut all_positions: Vec<Vec<(usize, String)>>) -> Vec<(usize, String)> {
    let mut determined_positions = Vec::new();
    let mut taken = HashSet::new();
    all_positions.sort_unstable_by(|a, b| a.len().cmp(&b.len()));
    for (i, positions) in all_positions.into_iter().enumerate() {
        if positions.len() != i + 1 {
            panic!("too big!");
        }
        for (position, name) in positions {
            if !taken.contains(&position) {
                determined_positions.push((position, name));
                taken.insert(position);
                break;
            }
        }
    }
    determined_positions
}

#[cfg(test)]
mod tests {
    use super::*;

    use indoc::indoc;

    const TEST_INPUT: &str = indoc! {"
        class: 1-3 or 5-7
        row: 6-11 or 33-44
        seat: 13-40 or 45-50
        
        your ticket:
        7,1,14
        
        nearby tickets:
        7,3,47
        40,4,50
        55,2,20
        38,6,12
    "};

    const TEST_INPUT_TWO: &str = indoc! {"
        class: 0-1 or 4-19
        row: 0-5 or 8-19
        seat: 0-13 or 16-19

        your ticket:
        11,12,13

        nearby tickets:
        3,9,18
        15,1,5
        5,14,9
    "};

    #[test]
    fn it_collects_correct_invalid_fields() {
        let mut lines = TEST_INPUT.lines();
        let rules = read_rules(&mut lines);

        read_passports(&mut lines); // read my passport
        let near_passports = read_passports(&mut lines);
        let invalid_fields = find_all_invalid_fields(&near_passports, &rules);
        assert_eq!(invalid_fields, vec![&4, &55, &12]);
    }

    #[test]
    fn it_collects_valid_positions() {
        let mut lines = TEST_INPUT_TWO.lines();
        let rules = read_rules(&mut lines);

        read_passports(&mut lines); // read my passport
        let near_passports = read_passports(&mut lines);
        let valid_passports = filter_invalid(near_passports, &rules);
        let valid_positions = find_all_valid_positions(&rules, &valid_passports);
        assert_eq!(
            valid_positions,
            vec![
                vec![(1, "class".to_string()), (2, "class".to_string())],
                vec![
                    (0, "row".to_string()),
                    (1, "row".to_string()),
                    (2, "row".to_string())
                ],
                vec![(2, "seat".to_string())]
            ]
        );
    }

    #[test]
    fn it_determines_valid_positions() {
        let mut lines = TEST_INPUT_TWO.lines();
        let rules = read_rules(&mut lines);

        read_passports(&mut lines); // read my passport
        let near_passports = read_passports(&mut lines);
        let valid_passports = filter_invalid(near_passports, &rules);
        let valid_positions = find_all_valid_positions(&rules, &valid_passports);
        let determined = determine_field_positions(valid_positions);
        assert_eq!(
            determined,
            vec![
                (2, "seat".to_string()),
                (1, "class".to_string()),
                (0, "row".to_string())
            ]
        );
    }
}
