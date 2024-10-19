mod emojis;
use emojis::get_emoji;

use nu_ansi_term::{AnsiStrings as ANSIStrings, Color, Style as ANSIStyle};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyTuple};
use regex::Regex;
use std::io::{self, Write};
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};
use tabled::grid::config::{Entity, Formatting};
use tabled::{
    builder::Builder,
    settings::{object::Rows, Alignment, Style},
};

fn extract_table_data(pylist: &PyList) -> PyResult<Vec<Vec<String>>> {
    let mut data = Vec::new();
    for item in pylist.iter() {
        if let Ok(row) = item.downcast::<PyList>() {
            let mut row_data = Vec::new();
            for cell in row.iter() {
                let cell_str = cell.str()?.to_string();
                row_data.push(cell_str);
            }
            data.push(row_data);
        } else {
            return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "Expected a list of lists for table data.",
            ));
        }
    }
    Ok(data)
}

fn render_table_string(
    data: &[Vec<String>],
    default_number_style: Option<ANSIStyle>,
) -> PyResult<String> {
    let mut builder = Builder::default();
    for row in data {
        let styled_row: Vec<String> = row
            .iter()
            .map(
                |cell| match parse_and_style_text(cell, default_number_style.clone()) {
                    Ok(styled_cell) => styled_cell,
                    Err(_) => cell.clone(),
                },
            )
            .collect();
        builder.push_record(styled_row);
    }
    let mut table = builder.build();

    // Set the table style
    table
        .with(Style::modern())
        .modify(Rows::new(1..), Alignment::center());

    // Enable ANSI escape codes using formatting configuration
    let formatting = Formatting::new(false, false, false); // This allows ANSI codes
    table
        .get_config_mut()
        .set_formatting(Entity::Global, formatting);

    Ok(table.to_string())
}

fn is_list_of_lists(pylist: &PyList) -> bool {
    pylist.iter().all(|item| item.downcast::<PyList>().is_ok())
}

#[pyfunction]
#[pyo3(signature = (*args, **kwargs))]
// Fast print function supporting styles, tables, and highlighted code.
//
// Parameters:
//     *args: Variable length argument list to be printed.
//     **kwargs: Keyword arguments (sep, end) similar to Python's print function.
//
// Usage:
//     print("Hello, [bold]World[/bold]!", sep=" - ", end="!\n")
//
//     data = [["Name", "Age"], ["Alice", "30"]]
//     print(data)
fn rprint(args: &PyTuple, kwargs: Option<&PyDict>) -> PyResult<()> {
    // Handle keyword arguments similar to Python's print function
    let sep = kwargs
        .and_then(|dict| dict.get_item("sep"))
        .and_then(|item| item.extract::<&str>().ok())
        .unwrap_or(" ");

    let end = kwargs
        .and_then(|dict| dict.get_item("end"))
        .and_then(|item| item.extract::<&str>().ok())
        .unwrap_or("\n");

    // Get number_style keyword argument
    let number_style_str = kwargs
        .and_then(|dict| dict.get_item("number_style"))
        .and_then(|item| item.extract::<&str>().ok());

    // Parse number_style into ANSIStyle
    let default_number_style = if let Some(style_str) = number_style_str {
        parse_style(ANSIStyle::new(), style_str).ok()
    } else {
        // Use bright_yellow by default
        parse_style(ANSIStyle::new(), "bright_yellow").ok()
    };

    // Regex to detect ANSI escape codes
    let ansi_regex = Regex::new(r"\x1B\[[0-9;]*[a-zA-Z]").unwrap();

    let mut output = String::new();

    // Process each argument
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            output.push_str(sep);
        }

        let arg_str = if let Ok(s) = arg.extract::<&str>() {
            if ansi_regex.is_match(s) {
                // String contains ANSI codes; skip processing
                s.to_string()
            } else {
                // Apply styling to string arguments
                match parse_and_style_text(s, default_number_style.clone()) {
                    Ok(styled_text) => styled_text,
                    Err(_) => s.to_string(), // If parsing fails, use the original string
                }
            }
        } else if let Ok(s) = arg.str() {
            let s_str = s.to_string();
            if ansi_regex.is_match(&s_str) {
                // String contains ANSI codes; skip processing
                s_str
            } else {
                match parse_and_style_text(&s_str, default_number_style.clone()) {
                    Ok(styled_text) => styled_text,
                    Err(_) => s_str,
                }
            }
        } else if let Ok(list) = arg.downcast::<PyList>() {
            // Check if the list is a list of lists
            if is_list_of_lists(list) {
                // Treat as table data
                let data = extract_table_data(list)?;
                let table_str = render_table_string(&data, default_number_style.clone())?;
                table_str
            } else {
                // Convert the list to a string representation
                let list_str = format!("{:?}", list);
                list_str
            }
        } else {
            arg.str()?.to_string()
        };

        output.push_str(&arg_str);
    }

    output.push_str(end);

    // Use Rust's stdout for efficient printing
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    handle.write_all(output.as_bytes())?;
    handle.flush()?;

    Ok(())
}

/// Highlight code with syntax highlighting.
#[pyfunction]
fn highlight(code: &str, language: &str) -> PyResult<String> {
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    // Find the syntax for the given language.
    let syntax = ps
        .find_syntax_by_token(language)
        .ok_or_else(|| PyValueError::new_err(format!("Language '{}' not supported", language)))?;

    let mut h = HighlightLines::new(syntax, &ts.themes["InspiredGitHub"]);

    let mut highlighted = String::new();
    for line in LinesWithEndings::from(code) {
        let ranges = h
            .highlight_line(line, &ps)
            .map_err(|e| PyValueError::new_err(format!("Highlighting error: {}", e)))?;
        let escaped = as_24_bit_terminal_escaped(&ranges[..], true);
        highlighted.push_str(&escaped);
    }

    // Append ANSI reset code to reset terminal styles
    highlighted.push_str("\x1b[0m");

    Ok(highlighted)
}

#[pyfunction]
fn style_text(text: &str) -> PyResult<String> {
    let styled_text = parse_and_style_text(text, None)?;
    Ok(styled_text)
}

// Parse a style string and return an ANSIStyle.
fn parse_and_style_text(
    text: &str,
    default_number_style: Option<ANSIStyle>,
) -> Result<String, PyErr> {
    // Regular expressions to match tags and emoji placeholders
    let tag_pattern = r"(?P<open>\[/(?P<close_style>[^\[\]/]+)\]|\[(?P<style>[^\[\]/]+)\])";
    let emoji_pattern = r":(?P<emoji_name>[a-zA-Z0-9_]+):";
    let combined_pattern = format!("{}|{}", tag_pattern, emoji_pattern);
    let combined_regex = Regex::new(&combined_pattern).unwrap();

    // Regex to match numbers
    let number_regex = Regex::new(r"\d+").unwrap();

    let mut output = Vec::new();
    let mut style_stack: Vec<ANSIStyle> = Vec::new();
    let mut last_index = 0;

    for mat in combined_regex.find_iter(text) {
        // Add text before the match
        if mat.start() > last_index {
            let segment = &text[last_index..mat.start()];
            let current_style = style_stack.last().cloned().unwrap_or_default();

            if style_stack.is_empty() {
                // No styles are active, so we can apply default number styling
                let mut last_num_end = 0;
                for num_match in number_regex.find_iter(segment) {
                    // Text before the number
                    if num_match.start() > last_num_end {
                        let before_num = &segment[last_num_end..num_match.start()];
                        output.push(current_style.paint(before_num));
                    }

                    // The number
                    let num_str = &segment[num_match.start()..num_match.end()];
                    if let Some(number_style) = &default_number_style {
                        output.push(number_style.paint(num_str));
                    } else {
                        output.push(current_style.paint(num_str));
                    }

                    last_num_end = num_match.end();
                }
                // Text after the last number
                if last_num_end < segment.len() {
                    let after_num = &segment[last_num_end..];
                    output.push(current_style.paint(after_num));
                }
            } else {
                // Styles are active, so we use the current style
                output.push(current_style.paint(segment));
            }
        }

        let matched_text = mat.as_str();

        if let Some(caps) = combined_regex.captures(matched_text) {
            if let Some(style_str) = caps.name("style") {
                // Opening tag
                let current_style = style_stack.last().cloned().unwrap_or_default();
                let new_style = parse_style(current_style, style_str.as_str())?;
                style_stack.push(new_style);
            } else if caps.name("close_style").is_some() {
                // Closing tag
                if style_stack.pop().is_none() {
                    return Err(PyValueError::new_err("Unmatched closing tag"));
                }
            } else if let Some(emoji_caps) = caps.name("emoji_name") {
                // Emoji placeholder
                let emoji_name = emoji_caps.as_str();
                let emoji_char = get_emoji(emoji_name);
                let current_style = style_stack.last().cloned().unwrap_or_default();
                output.push(current_style.paint(emoji_char));
            }
        }

        last_index = mat.end();
    }

    // Add any remaining text after the last match
    if last_index < text.len() {
        let segment = &text[last_index..];
        let current_style = style_stack.last().cloned().unwrap_or_default();

        if style_stack.is_empty() {
            // No styles are active, so we can apply default number styling
            let mut last_num_end = 0;
            for num_match in number_regex.find_iter(segment) {
                // Text before the number
                if num_match.start() > last_num_end {
                    let before_num = &segment[last_num_end..num_match.start()];
                    output.push(current_style.paint(before_num));
                }

                // The number
                let num_str = &segment[num_match.start()..num_match.end()];
                if let Some(number_style) = &default_number_style {
                    output.push(number_style.paint(num_str));
                } else {
                    output.push(current_style.paint(num_str));
                }

                last_num_end = num_match.end();
            }
            // Text after the last number
            if last_num_end < segment.len() {
                let after_num = &segment[last_num_end..];
                output.push(current_style.paint(after_num));
            }
        } else {
            // Styles are active, so we use the current style
            output.push(current_style.paint(segment));
        }
    }

    // Ensure terminal styles are reset at the end
    output.push(ANSIStyle::new().paint("\x1b[0m"));

    Ok(ANSIStrings(&output).to_string())
}

/// Parse a color name and return a Color.
fn parse_color(color_name: &str) -> Option<Color> {
    match color_name.to_lowercase().as_str() {
        "black" => Some(Color::Black),
        "red" => Some(Color::Red),
        "green" => Some(Color::Green),
        "yellow" => Some(Color::Yellow),
        "blue" => Some(Color::Blue),
        "magenta" | "purple" => Some(Color::Magenta),
        "cyan" => Some(Color::Cyan),
        "white" => Some(Color::White),
        "bright_black" | "grey" | "gray" => Some(Color::Fixed(237)),
        "bright_red" => Some(Color::LightRed),
        "bright_green" => Some(Color::LightGreen),
        "bright_yellow" => Some(Color::LightYellow),
        "bright_blue" => Some(Color::LightBlue),
        "bright_magenta" | "bright_purple" => Some(Color::LightMagenta),
        "bright_cyan" => Some(Color::LightCyan),
        "bright_white" => Some(Color::Fixed(252)),
        _ => None,
    }
}

// Parse styles from a string
fn parse_style(base_style: ANSIStyle, style_str: &str) -> Result<ANSIStyle, PyErr> {
    let mut style = base_style;
    for token in style_str.split_whitespace() {
        match token.to_lowercase().as_str() {
            // Text attributes
            "bold" | "b" => style = style.bold(),
            "dim" | "dimmed" => style = style.dimmed(),
            "italic" | "i" => style = style.italic(),
            "underline" | "u" => style = style.underline(),
            "blink" => style = style.blink(),
            "reverse" => style = style.reverse(),
            "hidden" => style = style.hidden(),
            "strikethrough" => style = style.strikethrough(),
            // Foreground colors
            "black" | "red" | "green" | "yellow" | "blue" | "magenta" | "purple" | "cyan"
            | "white" | "bright_black" | "grey" | "gray" | "bright_red" | "bright_green"
            | "bright_yellow" | "bright_blue" | "bright_magenta" | "bright_purple"
            | "bright_cyan" | "bright_white" => {
                if let Some(color) = parse_color(token) {
                    style = style.fg(color);
                } else {
                    return Err(PyValueError::new_err(format!("Invalid color: '{}'", token)));
                }
            }
            // Background colors (starting with 'on_')
            _ if token.starts_with("on_") => {
                let color_name = &token[3..];
                if let Some(color) = parse_color(color_name) {
                    style = style.on(color);
                } else {
                    return Err(PyValueError::new_err(format!(
                        "Invalid background color: '{}'",
                        color_name
                    )));
                }
            }
            // Invalid tokens
            _ => {
                return Err(PyValueError::new_err(format!(
                    "Invalid style token: '{}'",
                    token
                )));
            }
        }
    }
    Ok(style)
}

#[pyfunction]
fn render_table(data: Vec<Vec<String>>) -> PyResult<String> {
    let default_number_style = None; // Or set a default style if desired
    let table_str = render_table_string(&data, default_number_style)?;
    Ok(table_str)
}

/// A Python module implemented in Rust.
#[pymodule]
#[pyo3(name = "luxeprint")]
fn luxeprint(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(render_table, m)?)?;
    m.add_function(wrap_pyfunction!(highlight, m)?)?;
    m.add_function(wrap_pyfunction!(style_text, m)?)?;
    m.add_function(wrap_pyfunction!(rprint, m)?)?;
    Ok(())
}
