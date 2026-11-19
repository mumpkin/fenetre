//! Compiler messages, read from glslang's text output.

use std::fmt;

/// How serious a [`Diagnostic`] is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    /// The shader does not compile.
    Error,
    /// The shader compiles, but something looks wrong.
    Warning,
}

/// One message from the compiler, located in the user's source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    /// Error or warning.
    pub severity: Severity,
    /// 1-based line in the user's source, when glslang gives one.
    pub line: Option<u32>,
    /// 1-based column (in characters) of the token the message is about,
    /// when that token can be found on the line.
    pub column: Option<u32>,
    /// Readable message, e.g. `'foo': undeclared identifier`.
    pub message: String,
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(line) = self.line {
            write!(f, "{line}:")?;
            if let Some(column) = self.column {
                write!(f, "{column}:")?;
            }
            f.write_str(" ")?;
        }
        let label = match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };
        write!(f, "{label}: {}", self.message)
    }
}

/// Reads glslang's output (`name:LINE: error: 'token' : message` per line)
/// and finds columns in `user_source`. A `name: text` line without a
/// severity continues the previous message (the linker does that). Other
/// lines, such as `2 errors generated.`, are dropped, as is the cascade
/// message `compilation terminated`.
pub(crate) fn parse(output: &str, file_name: &str, user_source: &str) -> Vec<Diagnostic> {
    let source_lines: Vec<&str> = user_source.lines().collect();
    let mut messages: Vec<RawMessage> = Vec::new();
    for line in output.lines() {
        if let Some(raw) = parse_line(line, file_name) {
            messages.push(raw);
        } else if let (Some(more), Some(last)) =
            (continuation(line, file_name), messages.last_mut())
        {
            last.text = format!("{} {more}", last.text);
        }
    }
    messages
        .into_iter()
        .filter(|raw| raw.text != "compilation terminated")
        .map(|raw| {
            let column = raw
                .line
                .and_then(|n| source_lines.get(usize::try_from(n).ok()?.checked_sub(1)?))
                .zip(raw.token.as_deref())
                .and_then(|(text, token)| find_token(text, token));
            Diagnostic {
                severity: raw.severity,
                line: raw.line,
                column,
                message: match raw.token {
                    Some(token) => format!("'{token}': {}", raw.text),
                    None => raw.text,
                },
            }
        })
        .collect()
}

struct RawMessage {
    severity: Severity,
    line: Option<u32>,
    token: Option<String>,
    text: String,
}

fn parse_line(line: &str, file_name: &str) -> Option<RawMessage> {
    let rest = line.strip_prefix(file_name)?.strip_prefix(':')?;
    let (line_number, rest) = match rest.split_once(':') {
        Some((n, rest)) if n.trim().parse::<u32>().is_ok() => (n.trim().parse().ok(), rest),
        _ => (None, rest),
    };
    let rest = rest.trim_start();
    let (severity, body) = if let Some(body) = rest.strip_prefix("error:") {
        (Severity::Error, body)
    } else {
        (Severity::Warning, rest.strip_prefix("warning:")?)
    };
    let (token, text) = split_token(body.trim());
    Some(RawMessage {
        severity,
        line: line_number,
        token,
        text: squeeze_spaces(text),
    })
}

/// Text of a `name:    text` line. A mangled function signature such as
/// `mainImage(vf4;vf2;` is cut down to its name.
fn continuation<'a>(line: &'a str, file_name: &str) -> Option<&'a str> {
    let text = line.strip_prefix(file_name)?.strip_prefix(':')?.trim();
    let text = text.split_once('(').map_or(text, |(name, _)| name);
    (!text.is_empty()).then_some(text)
}

/// Splits `'token' : message` into its parts. An empty token gives `None`.
fn split_token(body: &str) -> (Option<String>, &str) {
    let Some(after_quote) = body.strip_prefix('\'') else {
        return (None, body);
    };
    let Some((token, rest)) = after_quote.split_once('\'') else {
        return (None, body);
    };
    let Some(text) = rest.trim_start().strip_prefix(':') else {
        return (None, body);
    };
    let token = (!token.is_empty()).then(|| token.to_owned());
    (token, text.trim())
}

fn squeeze_spaces(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Finds `token` on `line` as a whole word and returns its 1-based column.
fn find_token(line: &str, token: &str) -> Option<u32> {
    let is_word = |c: char| c.is_alphanumeric() || c == '_';
    let wordy = token.chars().all(is_word);
    let mut from = 0;
    while let Some(found) = line[from..].find(token) {
        let start = from + found;
        let end = start + token.len();
        let before_ok = !wordy || !line[..start].chars().next_back().is_some_and(is_word);
        let after_ok = !wordy || !line[end..].chars().next().is_some_and(is_word);
        if before_ok && after_ok {
            return u32::try_from(line[..start].chars().count() + 1).ok();
        }
        from = end;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: &str = "void main() {\n  c = vec4(undefinedThing, 1.0);\n  int x = 1.5;\n}\n";

    #[test]
    fn reads_glslang_output() {
        let output = "\
shader.frag:2: error: 'undefinedThing' : undeclared identifier
shader.frag:2: error: 'assign' :  cannot convert from ' const float' to 'vec4'
shader.frag:3: error: '=' :  cannot convert from ' const float' to ' temp highp int'
shader.frag:5: error: '' :  syntax error, unexpected IDENTIFIER
shader.frag: error: '' : compilation terminated
4 errors generated.
";
        let diagnostics = parse(output, "shader.frag", SOURCE);
        assert_eq!(diagnostics.len(), 4);
        assert_eq!(
            diagnostics[0],
            Diagnostic {
                severity: Severity::Error,
                line: Some(2),
                column: Some(12),
                message: "'undefinedThing': undeclared identifier".to_owned(),
            }
        );
        assert_eq!(diagnostics[1].column, None, "'assign' is not in the source");
        assert_eq!(diagnostics[2].column, Some(9));
        assert_eq!(
            diagnostics[3].message,
            "syntax error, unexpected IDENTIFIER"
        );
        assert_eq!(diagnostics[3].column, None);
    }

    #[test]
    fn reads_warnings_and_lineless_messages() {
        let output = "shader.frag:1: warning: 'x' : something odd\nshader.frag: error: '' : bad\n";
        let diagnostics = parse(output, "shader.frag", "float x;");
        assert_eq!(diagnostics[0].severity, Severity::Warning);
        assert_eq!(diagnostics[0].column, Some(7));
        assert_eq!(diagnostics[1].line, None);
    }

    #[test]
    fn joins_linker_continuation_lines() {
        let output = "\
shader.frag: error: Linking fragment stage: No function definition (body) found:
shader.frag:     mainImage(vf4;vf2;
1 error generated.
";
        let diagnostics = parse(output, "shader.frag", "");
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].message,
            "Linking fragment stage: No function definition (body) found: mainImage"
        );
    }

    #[test]
    fn token_must_be_a_whole_word() {
        assert_eq!(find_token("float fooBar; foo = 1.0;", "foo"), Some(15));
        assert_eq!(find_token("a == b", "=="), Some(3));
    }

    #[test]
    fn displays_like_a_compiler() {
        let diagnostic = Diagnostic {
            severity: Severity::Error,
            line: Some(4),
            column: Some(2),
            message: "oops".to_owned(),
        };
        assert_eq!(diagnostic.to_string(), "4:2: error: oops");
    }
}
