//! Tab stops inside a SQL snippet.
//!
//! `${1:table_name}` is an editor stop. Bare `$1` stays a PostgreSQL parameter.
//! Stops that share an index are mirrors: editing one rewrites the others.

#[derive(Debug, Clone, PartialEq, Eq)]
struct Stop {
    index: u16,
    start: usize,
    end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnippetSession {
    stops: Vec<Stop>,
    /// Tab order. `$0` is last. Positive indexes are sorted.
    order: Vec<u16>,
    active: usize,
}

pub enum SnippetMove {
    Select(usize, usize),
    Finished,
}

impl SnippetSession {
    /// Expand a template. Ranges are relative to the returned text.
    /// `None` when the template has no tab stops.
    // cc-scan:allow LONG_FUNCTION — linear pipeline — one cohesive pass
    pub fn expand(template: &str) -> (String, Option<Self>) {
        let mut out = String::with_capacity(template.len());
        let mut stops = Vec::new();
        let bytes = template.as_bytes();
        let mut index = 0;
        while index < bytes.len() {
            if bytes[index] != b'$' {
                let Some(ch) = template[index..].chars().next() else {
                    break;
                };
                out.push(ch);
                index += ch.len_utf8();
                continue;
            }
            if let Some((stop_index, placeholder, next)) = parse_braced_stop(template, index) {
                let start = out.len();
                out.push_str(placeholder);
                stops.push(Stop {
                    index: stop_index,
                    start,
                    end: out.len(),
                });
                index = next;
                continue;
            }
            if let Some(next) = parse_sql_parameter(template, index) {
                out.push_str(&template[index..next]);
                index = next;
                continue;
            }
            out.push('$');
            index += 1;
        }
        if stops.is_empty() {
            return (out, None);
        }
        let mut order: Vec<u16> = stops
            .iter()
            .map(|stop| stop.index)
            .filter(|index| *index != 0)
            .collect();
        order.sort_unstable();
        order.dedup();
        if stops.iter().any(|stop| stop.index == 0) {
            order.push(0);
        }
        (
            out,
            Some(Self {
                stops,
                order,
                active: 0,
            }),
        )
    }

    pub fn translate(&mut self, delta: usize) {
        for stop in &mut self.stops {
            stop.start += delta;
            stop.end += delta;
        }
    }

    pub fn active_range(&self) -> Option<(usize, usize)> {
        let index = *self.order.get(self.active)?;
        self.stops
            .iter()
            .find(|stop| stop.index == index)
            .map(|stop| (stop.start, stop.end))
    }

    pub fn move_by(&mut self, delta: i32) -> SnippetMove {
        if self.order.is_empty() {
            return SnippetMove::Finished;
        }
        if delta > 0 {
            if self.active + 1 >= self.order.len() {
                return SnippetMove::Finished;
            }
            self.active += 1;
        } else if self.active > 0 {
            self.active -= 1;
        }
        self.active_range()
            .map(|(start, end)| SnippetMove::Select(start, end))
            .unwrap_or(SnippetMove::Finished)
    }

    /// Apply the buffer's latest edit and rewrite mirrored stops.
    /// Returns false when the edit left the active stop and the session should end.
    pub fn note_edit(&mut self, buffer: &mut super::buffer::TextBuffer) -> bool {
        let Some((start, old_end, new_end)) = buffer.last_edit else {
            return true;
        };
        let Some(active_index) = self.order.get(self.active).copied() else {
            return false;
        };
        let Some(active) = self
            .stops
            .iter()
            .find(|stop| stop.index == active_index && stop.start <= start && stop.end >= old_end)
            .cloned()
        else {
            return false;
        };
        self.shift_edit(start, old_end, new_end, Some(active.start));
        let Some((active_start, active_end)) = self
            .stops
            .iter()
            .find(|stop| stop.index == active_index && stop.start == active.start)
            .map(|stop| (stop.start, stop.end))
        else {
            return false;
        };
        let new_text = buffer.slice(active_start, active_end).to_owned();
        let mut mirrors: Vec<usize> = self
            .stops
            .iter()
            .enumerate()
            .filter(|(_, stop)| stop.index == active_index && !(stop.start == active_start && stop.end == active_end))
            .map(|(index, _)| index)
            .collect();
        mirrors.sort_by(|&left, &right| self.stops[right].start.cmp(&self.stops[left].start));
        for mirror in mirrors {
            let (mirror_start, mirror_end) = (self.stops[mirror].start, self.stops[mirror].end);
            if buffer.slice(mirror_start, mirror_end) == new_text {
                continue;
            }
            buffer.replace(mirror_start, mirror_end, &new_text);
            self.shift_edit(mirror_start, mirror_end, mirror_start + new_text.len(), None);
        }
        true
    }

    // cc-scan:allow TOO_MANY_PARAMS — context params passed through
    fn shift_edit(&mut self, start: usize, old_end: usize, new_end: usize, edited_start: Option<usize>) {
        let delta = new_end as isize - old_end as isize;
        for stop in &mut self.stops {
            if edited_start == Some(stop.start) && stop.start <= start && stop.end >= old_end {
                stop.end = add_offset(stop.end, delta);
            } else if stop.start >= old_end {
                stop.start = add_offset(stop.start, delta);
                stop.end = add_offset(stop.end, delta);
            } else if stop.end > start && stop.start < old_end {
                stop.end = add_offset(stop.end, delta);
            }
        }
    }
}

fn add_offset(offset: usize, delta: isize) -> usize {
    if delta >= 0 {
        offset.saturating_add(delta as usize)
    } else {
        offset.saturating_sub((-delta) as usize)
    }
}

/// `${1:name}` or `${0}`. Returns the index, placeholder, and byte index after the stop.
fn parse_braced_stop(template: &str, dollar: usize) -> Option<(u16, &str, usize)> {
    let rest = template.get(dollar + 1..)?;
    if !rest.starts_with('{') {
        return None;
    }
    let body = &rest[1..];
    let digits = body.bytes().take_while(|byte| byte.is_ascii_digit()).count();
    if digits == 0 {
        return None;
    }
    let index: u16 = body[..digits].parse().ok()?;
    let after_digits = &body[digits..];
    if after_digits.starts_with('}') {
        return Some((index, "", dollar + 1 + 1 + digits + 1));
    }
    let placeholder = after_digits.strip_prefix(':')?;
    let close = placeholder.find('}')?;
    Some((index, &placeholder[..close], dollar + 1 + 1 + digits + 1 + close + 1))
}

/// `$1` or `$2`, not `${`.
fn parse_sql_parameter(template: &str, dollar: usize) -> Option<usize> {
    let rest = template.get(dollar + 1..)?;
    let digits = rest.bytes().take_while(|byte| byte.is_ascii_digit()).count();
    (digits > 0).then_some(dollar + 1 + digits)
}

#[cfg(test)]
#[path = "tests/snippet_tests.rs"]
mod tests;
