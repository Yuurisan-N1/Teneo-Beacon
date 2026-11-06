#![allow(dead_code)]

use std::io::IsTerminal;
use std::time::Duration;

use super::init::{blank, clear_live, emit, finish_live, live};

pub const G: &str = "\x1b[1;32m";
pub const Y: &str = "\x1b[1;33m";
pub const R: &str = "\x1b[1;31m";

pub fn lg(msg: &str) {
    emit(G, msg);
}

pub fn ly(msg: &str) {
    emit(Y, msg);
}

pub fn lr(msg: &str) {
    emit(R, msg);
}

pub fn gap() {
    blank();
}

pub fn status(msg: &str) {
    live(Y, msg);
}

pub fn clear_status() {
    clear_live();
}

#[allow(dead_code)]
pub fn finish_status() {
    finish_live();
}

pub fn squeeze(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut sp = false;
    for c in input.chars() {
        if c.is_control() || c == '\u{feff}' {
            if c == '\n' || c == '\r' || c == '\t' {
                sp = true;
            }
            continue;
        }
        if c.is_whitespace() {
            sp = true;
            continue;
        }
        if sp && !out.is_empty() {
            out.push(' ');
        }
        sp = false;
        out.push(c);
    }
    out
}

pub fn clean_text(value: &str, fallback: &str) -> String {
    let mut text = String::with_capacity(value.len());
    for c in value.chars() {
        if "[]|#!@$%^&*()-".contains(c) {
            text.push(' ');
        } else {
            text.push(c);
        }
    }
    let text = squeeze(&text);
    if text.is_empty() {
        fallback.to_string()
    } else {
        text
    }
}

pub fn mail(value: &str) -> String {
    let text = value.trim();
    if text.is_empty() {
        return "***".to_string();
    }
    match text.split_once('@') {
        Some((head, tail)) => {
            let keep: String = head.chars().take(2).collect();
            let stars = "*".repeat(head.chars().count().saturating_sub(2).max(3));
            format!("{}{}@{}", keep, stars, tail)
        }
        None => "***".to_string(),
    }
}

pub fn unit_word<'a>(value: u64, singular: &'a str, plural: &'a str) -> &'a str {
    if value == 1 {
        singular
    } else {
        plural
    }
}

pub fn clock(seconds: u64) -> String {
    let days = seconds / 86_400;
    let rest = seconds % 86_400;
    let hours = rest / 3_600;
    let minutes = (rest % 3_600) / 60;
    let secs = rest % 60;
    if days > 0 {
        format!("{:02}:{:02}:{:02}:{:02}", days, hours, minutes, secs)
    } else {
        format!("{:02}:{:02}:{:02}", hours, minutes, secs)
    }
}

pub fn num(value: f64) -> String {
    if !value.is_finite() {
        return "0".to_string();
    }
    let text = format!("{:.4}", value);
    let text = text.trim_end_matches('0').trim_end_matches('.');
    if text.is_empty() || text == "-" {
        return "0".to_string();
    }
    text.to_string()
}

pub fn cash(value: f64, unit: &str) -> String {
    format!("{} {}", num(value), unit)
}

pub fn countdown(seconds: u64, label: &str) {
    if seconds == 0 {
        return;
    }
    let text = clean_text(label, "Next cycle starts in");
    if !std::io::stdout().is_terminal() {
        ly(&format!("{} {}", text, clock(seconds)));
        std::thread::sleep(Duration::from_secs(seconds));
        return;
    }
    let mut left = seconds;
    while left > 0 {
        status(&format!("{} {}", text, clock(left)));
        std::thread::sleep(Duration::from_secs(1));
        left -= 1;
    }
    clear_status();
}

pub fn describe(err: &anyhow::Error) -> String {
    crate::support::error::reason(err)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_text_drops_banned_symbols() {
        assert_eq!(clean_text("Ad Task #1", "task"), "Ad Task 1");
        assert_eq!(clean_text("###", "task 7"), "task 7");
    }

    #[test]
    fn unit_word_pluralises() {
        assert_eq!(unit_word(1, "node is", "nodes are"), "node is");
        assert_eq!(unit_word(4, "node is", "nodes are"), "nodes are");
    }

    #[test]
    fn clock_uses_two_shapes() {
        assert_eq!(clock(3_723), "01:02:03");
        assert_eq!(clock(90_061), "01:01:01:01");
    }

    #[test]
    fn mail_keeps_two_letters_and_the_domain() {
        assert_eq!(mail("someone@gmail.com"), "so*****@gmail.com");
        assert_eq!(mail(""), "***");
    }

    #[test]
    fn num_trims_the_tail() {
        assert_eq!(num(1240.0), "1240");
        assert_eq!(num(f64::NAN), "0");
    }
}
