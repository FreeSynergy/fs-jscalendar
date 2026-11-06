//! Splitting an `.ics` stream into per-UID parts of one component kind.
//!
//! Pattern: Iterator-style scan — one pass over the lines; every part keeps the exact source
//! text (header, `VTIMEZONE`s, the UID's components) so the original can be stored byte for byte (ADR 0030).

use std::collections::BTreeMap;

/// Which objects a caller owns: its components become parts, the other kind stays out of them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Component {
    /// `VEVENT`, owned by `fs-calendar`.
    Event,
    /// `VTODO`, owned by `fs-tasks`.
    Task,
}

impl Component {
    fn name(self) -> &'static str {
        match self {
            Self::Event => "VEVENT",
            Self::Task => "VTODO",
        }
    }

    fn other(self) -> &'static str {
        match self {
            Self::Event => "VTODO",
            Self::Task => "VEVENT",
        }
    }
}

/// The part of an `.ics` stream that belongs to one `UID` (master + its overrides, with the
/// calendar header and all `VTIMEZONE`s).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IcsPart {
    /// The `UID` shared by the part's components.
    pub uid: String,
    /// Exact source text: the whole input if it holds one UID, otherwise header + `VTIMEZONE`s +
    /// the UID's components + `END:VCALENDAR`, every line copied unchanged.
    pub text: String,
}

/// One content line, unfolded (RFC 5545 3.1), and the physical lines it was written on.
struct Logical {
    first: usize,
    last: usize,
    text: String,
}

/// A line that begins with a space or tab continues the one before, whatever it says.
fn is_continuation(line: &str) -> bool {
    line.starts_with([' ', '\t'])
}

fn logical_lines(lines: &[&str]) -> Vec<Logical> {
    let mut out: Vec<Logical> = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        match out.last_mut().filter(|_| is_continuation(line)) {
            Some(open) => {
                open.last = i;
                open.text.push_str(line[1..].trim_end_matches(['\r', '\n']));
            }
            None => out.push(Logical {
                first: i,
                last: i,
                text: line.trim_end_matches(['\r', '\n']).to_string(),
            }),
        }
    }
    for line in &mut out {
        line.text.truncate(line.text.trim_end().len());
    }
    out
}

/// The content lines of an iCalendar text, unfolded. What looks like a property or a component on a
/// continuation line is part of the line before it.
pub fn unfolded_lines(text: &str) -> Vec<String> {
    let lines: Vec<&str> = text.split_inclusive('\n').collect();
    logical_lines(&lines).into_iter().map(|l| l.text).collect()
}

fn block_name(line: &str) -> Option<String> {
    let name = line
        .get(..6)
        .filter(|b| b.eq_ignore_ascii_case("BEGIN:"))
        .map(|_| &line[6..])?;
    (!name.is_empty()).then(|| name.to_ascii_uppercase())
}

fn uid_of(block: &[Logical]) -> Option<String> {
    let line = block.iter().find(|l| {
        l.text
            .get(..4)
            .is_some_and(|n| n.eq_ignore_ascii_case("UID:") || n.eq_ignore_ascii_case("UID;"))
    })?;
    let colon = line.text.find(':')?;
    Some(line.text[colon + 1..].to_string())
}

/// Split an `.ics` text into per-UID parts of the components `own` names.
pub fn split_ics(text: &str, own: Component) -> Vec<IcsPart> {
    let eol = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let lines: Vec<&str> = text.split_inclusive('\n').collect();
    let (mut head, mut tz): (Vec<&str>, Vec<&str>) = (vec![], vec![]);
    let mut comps: Vec<(String, Vec<&str>)> = vec![]; // (uid, lines) in order
    let logical = logical_lines(&lines);
    let mut i = 0;
    let mut seen_component = false;
    let mut foreign = false; // the other kind has its own owner; its text is not part of our original
    while i < logical.len() {
        let line = &logical[i];
        match block_name(&line.text).as_deref() {
            Some("VCALENDAR") | None => {
                if !line.text.eq_ignore_ascii_case("END:VCALENDAR") && !seen_component {
                    head.extend_from_slice(&lines[line.first..=line.last]);
                }
                i += 1;
            }
            Some(name) => {
                let close = format!("END:{name}");
                let end = (i + 1..logical.len())
                    .find(|&j| logical[j].text.eq_ignore_ascii_case(&close))
                    .unwrap_or(logical.len() - 1);
                let block = &lines[line.first..=logical[end].last];
                if name == "VTIMEZONE" {
                    tz.extend_from_slice(block);
                } else if name == own.name() {
                    if let Some(uid) = uid_of(&logical[i..=end]) {
                        comps.push((uid, block.to_vec()));
                    }
                } else if name == own.other() {
                    foreign = true;
                }
                seen_component |= name != "VTIMEZONE";
                i = end + 1;
            }
        }
    }
    let mut by_uid: BTreeMap<String, Vec<&str>> = BTreeMap::new();
    let mut order = vec![];
    for (uid, b) in comps {
        if !by_uid.contains_key(&uid) {
            order.push(uid.clone());
        }
        by_uid.entry(uid).or_default().extend(b);
    }
    if order.len() == 1 && !foreign {
        return vec![IcsPart {
            uid: order.remove(0),
            text: text.to_string(),
        }];
    }
    order
        .into_iter()
        .map(|uid| {
            let mut t = String::new();
            head.iter()
                .chain(tz.iter())
                .chain(by_uid[&uid].iter())
                .for_each(|l| t.push_str(l));
            if !t.ends_with('\n') {
                t.push_str(eol);
            }
            t.push_str("END:VCALENDAR");
            t.push_str(eol);
            IcsPart { uid, text: t }
        })
        .collect()
}
