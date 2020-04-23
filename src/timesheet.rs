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

const PAUSE_TEXTS: [&str; 3] = ["lunch", "mittag", "pause"];
const TIME_FORMAT: &str = "%H:%M";
lazy_static! {
    static ref OVERRIDE_REGEX: regex::Regex = regex::Regex::new("\\((.*)\\)").unwrap();
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TimePoint {
    text: String,
    time: Time,
}

impl TimePoint {
    pub fn new(text: &str) -> Self {
        let time = OffsetDateTime::now_local().time();
        Self {
            time,
            text: format!("[{}] {}", time.format(TIME_FORMAT), text),
        }
    }
}

impl fmt::Display for TimePoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.text)
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
    OVERRIDE_REGEX
        .captures(&s)
        // index 0 is the entire string
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str())
        .unwrap_or(&s)
        .to_string()
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

    fn current(&self) -> &TimePoint {
        &self.times[self.selected]
    }

    fn grouped_times(&self) -> impl Iterator<Item = (String, Duration)> {
        self.times
            .iter()
            .chain(iter::once(&TimePoint::new("end")))
            .tuple_windows()
            .map(|(prev, next)| {
                (
                    prev.text.clone().splitn(2, " ").last().unwrap().to_string(),
                    next.time - prev.time,
                )
            })
            // Fold into a map to group by description.
            // I use a BTreeMap because I need a stable output order for the iterator
            // (otherwise the summary list will jump around on every input).
            .fold(collections::BTreeMap::new(), |mut map, (text, duration)| {
                *map.entry(effective_text(text))
                    .or_insert_with(Duration::zero) += duration;
                map
            })
            .into_iter()
            .filter(|(text, _)| !PAUSE_TEXTS.contains(&text.as_str()))
    }

    pub fn time_by_tasks(&self) -> String {
        self.grouped_times()
            .map(|(text, duration)| format!("{} {}", text, format_duration(&duration)))
            .join(" | ")
    }

    pub fn sum_as_str(&self) -> String {
        let total = self
            .grouped_times()
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
        let old_text = self.current().text.clone();
        let parts: Vec<_> = old_text.splitn(2, " ").collect();
        if parts.len() < 2 {
            self.remove_current();
            self.selected = self.selected.saturating_sub(1);
            return;
        }
        let current = &mut self.times[self.selected];
        // if we have a parse error, just keep the old time
        if let Ok(t) = Time::parse(parts[0].replace('[', "").replace(']', ""), TIME_FORMAT) {
            current.time = t;
        }
        current.text = format!("[{}] {}", current.time.format(TIME_FORMAT), parts[1]);
        self.times.sort_by_key(|t| t.time);
    }

    // noop for this
    fn toggle_current(&mut self) {}

    fn append_to_current(&mut self, chr: char) {
        self.times[self.selected].text.push(chr);
    }

    fn backspace(&mut self) {
        self.times[self.selected].text.pop();
    }
}
