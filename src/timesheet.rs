use super::listview::ListView;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_json::from_reader;
use std::{collections, default, fmt, fs, io, iter};
use time::{Duration, OffsetDateTime, Time};

pub struct TimeSheet {
    pub times: Vec<TimePoint>,
    pub selected: usize,
    pub register: Option<TimePoint>,
}

const MAIN_PAUSE_TEXT: &str = "pause";
const PAUSE_TEXTS: [&str; 4] = [MAIN_PAUSE_TEXT, "lunch", "mittag", "break"];

lazy_static! {
    static ref OVERRIDE_REGEX: regex::Regex = regex::Regex::new("\\[(.*)\\]").unwrap();
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TimePoint {
    text: String,
    time: Time,
}

impl TimePoint {
    pub fn new(text: &str) -> Self {
        Self {
            text: String::from(text),
            time: OffsetDateTime::now_local().time(),
        }
    }
}

impl fmt::Display for TimePoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}] {}", self.time.format("%H:%M"), self.text)
    }
}

impl default::Default for TimePoint {
    fn default() -> Self {
        TimePoint::new("")
    }
}

fn read_times(path: &str) -> Option<Vec<TimePoint>> {
    fs::File::open(path)
        .ok()
        .map(io::BufReader::new)
        .and_then(|r| from_reader(r).ok())
}

/**
 * If a time text contains "[something]",
 * only use the message inside the brackets.
 */
fn effective_text(s: String) -> String {
    let text = OVERRIDE_REGEX
        .captures(&s)
        // index 0 is the entire string
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str())
        .unwrap_or(&s);
    if PAUSE_TEXTS.contains(&text) {
        MAIN_PAUSE_TEXT
    } else {
        text
    }.to_string()
}

impl TimeSheet {
    pub fn open_or_create(path: &str) -> Self {
        Self {
            times: read_times(path).unwrap_or_else(|| vec![TimePoint::new("start")]),
            selected: 0,
            register: None,
        }
    }

    pub fn printable(&self) -> Vec<String> {
        self.times.iter().map(TimePoint::to_string).collect()
    }

    /**
     * Adjust the current time by `minutes` and round the result to a multiple of `minutes`.
     * This is so I can adjust in steps of 5 but still get nice, even numbers in the output.
     */
    pub fn shift_current(&mut self, minutes: i64) {
        let time = &mut self.times[self.selected].time;
        *time += Duration::minutes(minutes);
        *time -= Duration::minutes(time.minute() as i64 % 5)
    }

    fn current(&self) -> &TimePoint {
        &self.times[self.selected]
    }

    fn grouped_times(&self) -> impl Iterator<Item = (String, Duration)> {
        self.times
            .iter()
            .chain(iter::once(&TimePoint::new("end")))
            .tuple_windows()
            .map(|(prev, next)| (prev.text.clone(), next.time - prev.time))
            // Fold into a map to group by description.
            // I use a BTreeMap because I need a stable output order for the iterator
            // (otherwise the summary list will jump around on every input).
            .fold(collections::BTreeMap::new(), |mut map, (text, duration)| {
                *map.entry(effective_text(text))
                    .or_insert_with(Duration::zero) += duration;
                map
            })
            .into_iter()
    }

    pub fn time_by_tasks(&self) -> String {
        self.grouped_times()
            .map(|(text, duration)| format!("{}: {}", text, format_duration(&duration)))
            .join(" | ")
    }

    pub fn sum_as_str(&self) -> String {
        let total = self
            .grouped_times()
            .filter(|(text, _)| text != MAIN_PAUSE_TEXT)
            .fold(Duration::zero(), |total, (_, d)| total + d);
        format_duration(&total)
    }
}

fn format_duration(d: &Duration) -> String {
    format!("{}:{:02}", d.whole_hours(), d.whole_minutes().max(1) % 60)
}

impl ListView<TimePoint> for TimeSheet {
    fn selection_pointer(&mut self) -> &mut usize {
        &mut self.selected
    }

    fn list(&mut self) -> &mut Vec<TimePoint> {
        &mut self.times
    }

    fn register(&mut self) -> &mut Option<TimePoint> {
        &mut self.register
    }

    fn normal_mode(&mut self) {
        if self.current().text.is_empty() {
            self.remove_current();
            self.selected = self.selected.saturating_sub(1);
        }
        self.times.sort_by_key(|t| t.time);
    }

    fn append_to_current(&mut self, chr: char) {
        self.times[self.selected].text.push(chr);
    }

    fn backspace(&mut self) {
        self.times[self.selected].text.pop();
    }
}
