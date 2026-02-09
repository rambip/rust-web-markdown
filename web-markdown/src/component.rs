use std::collections::BTreeMap;

/// A custom non-native html element defined inside markdown.
///
/// ## Custom Component Naming Rules
///
/// Custom components are identified by their tag names, which must follow specific rules
/// to distinguish them from standard HTML tags:
///
/// ### Valid Custom Component Names
///
/// A tag name is considered a custom component if it meets any of these criteria:
///
/// 1. **Starts with an uppercase letter (A-Z)**
///    - Examples: `<MyComponent>`, `<Counter>`, `<DataTable>`
///    - No dash required for uppercase names
///
/// 2. **Starts with a lowercase letter (a-z) and contains at least one dash (-)**
///    - Examples: `<my-component>`, `<data-table>`, `<custom-counter>`
///    - The dash distinguishes these from standard HTML tags
///
/// This ensures that standard HTML tags (like `<div>`, `<span>`, `<p>`) are never confused
/// with custom components. Custom component tags support start tags, end tags, self-closing
/// tags, and tags with attributes (e.g., `<MyComponent key="value" name="test"/>`).
///
/// ### Examples
///
/// ```markdown
/// <!-- Valid custom components -->
/// <Counter initial="5"/>
/// <my-widget/>
/// <DataTable>content</DataTable>
/// <custom-box>
///   **Bold text inside custom component**
/// </custom-box>
///
/// <!-- NOT custom components (standard HTML) -->
/// <div>text</div>
/// <span>text</span>
/// <p>paragraph</p>
/// ```
#[derive(Debug, PartialEq)]
pub struct ComponentCall<'a> {
    /// Where in the larger document full_string starts.
    pub range_offset: usize,
    /// Full string which is parsed into this component.
    pub full_string: &'a str,
    /// Name from the parsed tag.
    pub name: &'a str,
    /// The attribute values may contain escape codes: it is up to to the consumer of this string to do un-escaping if required.
    pub attributes: BTreeMap<&'a str, &'a str>,
}

/// An html tag, used to create a custom component.
#[derive(Debug, PartialEq)]
pub enum CustomHtmlTag<'a> {
    /// <Component key="value"/>
    Inline(ComponentCall<'a>),
    /// <Component>
    Start(ComponentCall<'a>),
    /// </Component>
    End(&'a str),
}

type ParseError = String;

fn parse_attribute_value<'a>(stream: &mut &'a str) -> Result<&'a str, ParseError> {
    parse_expect_character(stream, '"', "please use `\"` to wrap your attribute values")?;

    match stream.split_once('"') {
        Some((content, stream_new)) => {
            *stream = stream_new;
            Ok(content)
        }
        None => Err("expected attribute value".into()),
    }
}

fn parse_expect_character(
    stream: &mut &str,
    expected: char,
    error_message: &str,
) -> Result<(), ParseError> {
    match check_and_skip(stream, expected) {
        true => Ok(()),
        false => Err(error_message.into()),
    }
}

fn check_and_skip(stream: &mut &str, expected: char) -> bool {
    if stream.starts_with(expected) {
        // Skip over expected
        *stream = &stream[1..];
        true
    } else {
        false
    }
}

/// Reads and trims an identifier up to an equals sign
///
/// Trailing "=" is read from the stream.
fn parse_attribute_name<'a>(stream: &mut &'a str) -> Result<&'a str, ParseError> {
    match stream.split_once('=') {
        Some((name, stream_new)) => {
            *stream = stream_new;
            let trimmed = name.trim();
            if trimmed.find(char::is_whitespace).is_some() {
                return Err(
                    "attribute name must be followed by equals sign, and not contain whitespace"
                        .into(),
                );
            }
            Ok(name.trim())
        }
        None => Err("expected equal sign after attribute name".into()),
    }
}

fn parse_attribute<'a>(stream: &mut &'a str) -> Result<(&'a str, &'a str), ParseError> {
    let name = parse_attribute_name(stream)?;
    // spaces
    *stream = stream.trim_start();
    let attribute = parse_attribute_value(stream)?;

    Ok((name, attribute))
}

#[derive(Debug)]
pub struct CustomHtmlTagError {
    /// The name, if one was parsed before erroring.
    pub name: Option<String>,
    /// THe error message.
    pub message: String,
}

impl CustomHtmlTag<'_> {
    /// Parse an Html Tag.
    /// This only supports the [Double-quoted attribute value syntax](https://www.w3.org/TR/2014/REC-html5-20141028/syntax.html#syntax-attributes)
    /// and does not robustly validate things like invalid characters in attribute names.
    pub fn from_str(
        s: &'_ str,
        range_offset: usize,
    ) -> Result<CustomHtmlTag<'_>, CustomHtmlTagError> {
        let mut s2 = s;
        let stream = &mut s2;
        parse_expect_character(stream, '<', "expected <").map_err(|e| CustomHtmlTagError {
            name: None,
            message: e,
        })?;

        let is_closing_tag = check_and_skip(stream, '/');

        let mut name = &stream[0..0];
        for (index, char) in stream.char_indices() {
            if char.is_whitespace() || char == '/' || char == '>' {
                name = &stream[0..index];
                *stream = &stream[index..];
                break;
            }
        }

        let err = {
            let name = name.to_string();
            move |message| -> Result<CustomHtmlTag, CustomHtmlTagError> {
                Err(CustomHtmlTagError {
                    name: Some(name.clone()),
                    message,
                })
            }
        };

        let mut attributes = BTreeMap::new();
        loop {
            *stream = stream.trim_start();
            match stream.chars().nth(0) {
                None => return err("expected end of tag".into()),
                Some('/') => {
                    return Ok(CustomHtmlTag::Inline(ComponentCall {
                        name,
                        attributes,
                        full_string: s,
                        range_offset,
                    }))
                }
                Some('>') => {
                    return if is_closing_tag {
                        Ok(CustomHtmlTag::End(name))
                    } else {
                        Ok(CustomHtmlTag::Start(ComponentCall {
                            name,
                            attributes,
                            full_string: s,
                            range_offset,
                        }))
                    }
                }
                _ => {
                    let parsed = parse_attribute(stream);
                    match parsed {
                        Ok((name, value)) => attributes.insert(name, value),
                        Err(message) => return err(message),
                    };
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use CustomHtmlTag::*;

    #[test]
    fn parse_start() {
        let full_string = "<a>";
        let c: CustomHtmlTag = CustomHtmlTag::from_str(full_string, 0).unwrap();
        assert_eq!(
            c,
            Start(ComponentCall {
                name: &full_string[1..2],
                attributes: [].into(),
                range_offset: 0,
                full_string
            },)
        )
    }

    #[test]
    fn parse_end() {
        let c: CustomHtmlTag = CustomHtmlTag::from_str("</a>", 0).unwrap();
        assert_eq!(c, End("a".into()))
    }

    #[test]
    fn parse_inline_empty() {
        let full_string = "<a/>";
        let c: CustomHtmlTag = CustomHtmlTag::from_str(full_string, 0).unwrap();
        assert_eq!(
            c,
            Inline(ComponentCall {
                name: &full_string[1..2],
                attributes: [].into(),
                range_offset: 0,
                full_string
            },)
        )
    }

    #[test]
    fn parse_inline() {
        let full_string = "<a key=\"val\"/>";
        let c: CustomHtmlTag = CustomHtmlTag::from_str(full_string, 1).unwrap();
        assert_eq!(
            c,
            Inline(ComponentCall {
                name: &full_string[1..2],
                attributes: BTreeMap::from([(&full_string[3..6], &full_string[8..11])]),
                range_offset: 1,
                full_string
            },)
        )
    }

    #[test]
    fn parse_error() {
        let c: Result<CustomHtmlTag, CustomHtmlTagError> = CustomHtmlTag::from_str("<a x>", 0);
        match c {
            Ok(_) => panic!(),
            Err(CustomHtmlTagError {
                name: Some(name),
                message: _,
            }) => assert_eq!(name, "a"),
            Err(CustomHtmlTagError {
                name: None,
                message: _,
            }) => panic!(),
        }
    }
}
