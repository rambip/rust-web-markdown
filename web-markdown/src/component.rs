use std::collections::BTreeMap;

#[derive(Debug, PartialEq)]
/// a custom non-native html element
/// called inside markdown
pub struct ComponentCall<'a> {
    pub name: &'a str,
    pub attributes: BTreeMap<&'a str, &'a str>,
}

#[derive(Debug, PartialEq)]
/// An html tag, used to create a custom component
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
            return Ok(content);
        }
        None => return Err("expected attribute value".into()),
    }
}

fn parse_expect_character<'a>(
    stream: &mut &'a str,
    expected: char,
    error_message: &str,
) -> Result<(), ParseError> {
    match check_and_skip(stream, expected) {
        true => Ok(()),
        false => Err(error_message.into()),
    }
}

fn check_and_skip<'a>(stream: &mut &'a str, expected: char) -> bool {
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
            return Ok(name.trim());
        }
        None => return Err("expected equal sign after attribute name".into()),
    }
}

fn parse_attribute<'a>(stream: &mut &'a str) -> Result<(&'a str, &'a str), ParseError> {
    let name = parse_attribute_name(stream)?;
    // spaces
    *stream = &stream.trim_start();
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
    pub fn from_str(s: &'_ str) -> Result<CustomHtmlTag<'_>, CustomHtmlTagError> {
        let mut s2 = s;
        let mut stream = &mut s2;
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
                Some('/') => return Ok(CustomHtmlTag::Inline(ComponentCall { name, attributes })),
                Some('>') => {
                    return if is_closing_tag {
                        Ok(CustomHtmlTag::End(name))
                    } else {
                        Ok(CustomHtmlTag::Start(ComponentCall { name, attributes }))
                    }
                }
                _ => {
                    let parsed = parse_attribute(&mut stream);
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
        let c: CustomHtmlTag = CustomHtmlTag::from_str("<a>").unwrap();
        assert_eq!(
            c,
            Start(ComponentCall {
                name: "a".into(),
                attributes: [].into(),
            },)
        )
    }

    #[test]
    fn parse_end() {
        let c: CustomHtmlTag = CustomHtmlTag::from_str("</a>").unwrap();
        assert_eq!(c, End("a".into()))
    }

    #[test]
    fn parse_inline_empty() {
        let c: CustomHtmlTag = CustomHtmlTag::from_str("<a/>").unwrap();
        assert_eq!(
            c,
            Inline(ComponentCall {
                name: "a".into(),
                attributes: [].into(),
            },)
        )
    }

    #[test]
    fn parse_inline() {
        let c: CustomHtmlTag = CustomHtmlTag::from_str("<a key=\"val\"/>").unwrap();
        assert_eq!(
            c,
            Inline(ComponentCall {
                name: "a".into(),
                attributes: BTreeMap::from([("key".into(), "val".into())])
            },)
        )
    }

    #[test]
    fn parse_error() {
        let c: Result<CustomHtmlTag, CustomHtmlTagError> = CustomHtmlTag::from_str("<a x>");
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
