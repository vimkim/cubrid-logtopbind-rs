use regex::Regex;

pub enum ParsedLine<'a> {
    QueryNo(&'a str),
    BindNull,
    Bind(&'a str),
    Query(&'a str),
    End,
    Filename(&'a str),
}

pub fn parse_line<'a>(
    line: &'a str,
    re_query_no: &Regex,
    re_bind_null: &Regex,
    re_bind: &Regex,
    re_query: &Regex,
    re_end: &Regex,
    re_filename: &Regex,
) -> Option<ParsedLine<'a>> {
    if let Some(caps) = re_query_no.captures(line) {
        // Note: using .get(1) is a bit safer than indexing.
        if let Some(m) = caps.get(1) {
            return Some(ParsedLine::QueryNo(m.as_str()));
        }
    } else if re_bind_null.is_match(line) {
        return Some(ParsedLine::BindNull);
    } else if let Some(mat) = re_bind.find(line) {
        // Everything after the match is the captured text.
        let captured_text = &line[mat.end()..];
        return Some(ParsedLine::Bind(captured_text));
    } else if let Some(caps) = re_query.captures(line) {
        if let Some(m) = caps.get(1) {
            return Some(ParsedLine::Query(m.as_str()));
        }
    } else if re_end.is_match(line) {
        return Some(ParsedLine::End);
    } else if let Some(caps) = re_filename.captures(line) {
        if let Some(m) = caps.get(1) {
            return Some(ParsedLine::Filename(m.as_str()));
        }
    }
    None
}
