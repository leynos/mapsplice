//! Tests for shell command splitting across protected and top-level separators.

use proptest::prelude::*;

use super::shell_commands;

fn protected_fragment() -> impl Strategy<Value = String> {
    ("[a-z0-9 ]{0,8}", "[a-z0-9 ]{0,8}", 1usize..=4).prop_map(|(left, right, depth)| {
        let escaped = format!("{left}\\;{right}\\&\\&{left}\\|\\|{right}");
        let mut grouped = format!("({left}; {right} && {left} || {right})");
        for _ in 1..depth {
            grouped = format!("({grouped}; {right})");
        }
        let mut substitution = format!("printf '%s' '{left}; && ||'");
        for _ in 0..depth {
            substitution = format!("$(printf '%s' \"{substitution}\")");
        }

        format!("'{left}; && ||' \"{right}; && ||\" {escaped} {grouped} \"{substitution}\"")
    })
}

fn top_level_separator() -> impl Strategy<Value = &'static str> {
    prop_oneof![Just(";"), Just("&&"), Just("||")]
}

fn shell_chain() -> impl Strategy<Value = (Vec<String>, Vec<&'static str>)> {
    prop::collection::vec(protected_fragment(), 1..=4).prop_flat_map(|commands| {
        let separator_count = commands.len() - 1;
        (
            Just(commands),
            prop::collection::vec(top_level_separator(), separator_count..=separator_count),
        )
    })
}

#[test]
fn nested_substitution_separators_stay_inside_the_assignment() {
    let line = concat!(
        "CARGO_ENCODED_RUSTFLAGS=\"$(filtered=$(bash -c ",
        "'set -euo pipefail; printf \\\"%s\\\" value')); ",
        "if [ -n \"$filtered\" ]; then printf '%s' \"$filtered\"; fi)\" ",
        "RUSTFLAGS=\"-D warnings -Zthreads=8\" cargo build"
    );

    assert_eq!(shell_commands(line), vec![line.to_owned()]);
}

#[test]
fn top_level_operators_split_commands() {
    assert_eq!(
        shell_commands("before; middle && next || after"),
        vec!["before", " middle ", " next ", " after"],
    );
}

#[test]
fn empty_input_and_empty_command_fragments_are_retained() {
    assert_eq!(shell_commands(""), vec![""]);
    assert_eq!(shell_commands("&& a; ||"), vec!["", " a", " ", ""]);
}

#[test]
fn dangling_escape_is_preserved_literally() {
    let line = "cargo build \\";

    assert_eq!(shell_commands(line), vec![line.to_owned()]);
}

#[test]
fn top_level_newlines_split_commands() {
    assert_eq!(shell_commands("before\nmiddle"), vec!["before", "middle"]);
}

#[test]
fn protected_and_escaped_newlines_do_not_split_commands() {
    let line = concat!(
        "\"first\nsecond\" $(printf 'third\nfourth') ",
        "before \\\n after"
    );

    assert_eq!(shell_commands(line), vec![line.to_owned()]);
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 96,
        .. ProptestConfig::default()
    })]

    #[test]
    fn generated_protected_separators_do_not_split_shell_commands(
        (fragments, separators) in shell_chain(),
    ) {
        let mut fragment_iterator = fragments.iter();
        let first = fragment_iterator
            .next()
            .expect("shell_chain always generates at least one fragment");
        let mut line = first.clone();
        let mut expected = vec![first.clone()];
        for (separator, fragment) in separators.iter().zip(fragment_iterator) {
            line.push(' ');
            line.push_str(separator);
            line.push(' ');
            line.push_str(fragment);
            expected
                .last_mut()
                .expect("the first fragment initializes expected commands")
                .push(' ');
            expected.push(format!(" {fragment}"));
        }

        prop_assert_eq!(shell_commands(&line), expected);
    }
}
