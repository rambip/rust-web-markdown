use core::iter::Peekable;
use std::str::FromStr;

use std::collections::BTreeMap;

#[derive(Debug, PartialEq)]
/// a custom non-native html element
/// called inside markdown
pub struct ComponentCall {
    pub name: String,
    pub attributes: BTreeMap<String, String>,
}

#[derive(Debug, PartialEq)]
/// An html tag, used to create a custom component
pub enum CustomHtmlTag {
    /// <Component key="value"/>
    Inline(ComponentCall),
    /// <Component>
    Start(ComponentCall),
    /// </Component>
    End(String),
}

type ParseError = String;

fn parse_attribute_value(stream: &mut Peekable<std::str::Chars>) -> Result<String, ParseError> {
    let mut attribute = String::new();

    if stream.next() != Some('"') {
        return Err("please use `\"` to wrap your attribute values".into());
    }

    loop {
        match stream.peek() {
            None => return Err("expected attribute value".into()),
            Some(&'"') => break,
            _ => attribute.push(stream.next().unwrap()),
        }
    }
    stream.next();

    Ok(attribute)
}

fn parse_attribute_name(stream: &mut Peekable<std::str::Chars>) -> Result<String, ParseError> {
    let mut name = String::new();

    while stream.peek() == Some(&' ') {
        stream.next();
    }

    loop {
        match stream.peek() {
            None => return Err("expected equal sign after attribute name".into()),
            Some(&'=') => break,
            _ => name.push(stream.next().unwrap()),
        }
    }

    Ok(name)
}

fn parse_attribute(stream: &mut Peekable<std::str::Chars>) -> Result<(String, String), ParseError> {
    let name = parse_attribute_name(stream)?;
    // equal sign
    stream.next();
    // spaces
    while stream.peek() == Some(&' ') {
        stream.next();
    }
    let attribute = parse_attribute_value(stream)?;

    Ok((name, attribute))
}

impl FromStr for CustomHtmlTag {
    type Err = String;

    fn from_str(s: &str) -> Result<CustomHtmlTag, Self::Err> {
        let mut stream = s.chars().peekable();

        if stream.next() != Some('<') {
            return Err("expected <".into());
        }

        let is_end = if stream.peek() == Some(&'/') {
            stream.next();
            true
        } else {
            false
        };

        let mut name = String::new();

        loop {
            match stream.peek() {
                Some(&' ') | Some(&'/') | Some(&'>') => break,
                _ => name.push(stream.next().unwrap()),
            }
        }

        let mut attributes = BTreeMap::new();
        loop {
            match stream.peek() {
                None => return Err("expected end of tag".into()),
                Some(&'>') | Some(&'/') => break,
                _ => {
                    let (name, value) = parse_attribute(&mut stream)?;
                    attributes.insert(name, value);
                }
            }
        }

        if stream.peek() == Some(&'/') {
            return Ok(CustomHtmlTag::Inline(ComponentCall {
                name,
                attributes: attributes.into(),
            }));
        }

        while stream.peek() == Some(&' ') {
            stream.next();
        }

        if is_end {
            Ok(CustomHtmlTag::End(name))
        } else {
            Ok(CustomHtmlTag::Start(ComponentCall { name, attributes }))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use CustomHtmlTag::*;

    #[test]
    fn parse_start() {
        let c: CustomHtmlTag = "<a>".parse().unwrap();
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
        let c: CustomHtmlTag = "</a>".parse().unwrap();
        assert_eq!(c, End("a".into()))
    }

    #[test]
    fn parse_inline_empty() {
        let c: CustomHtmlTag = "<a/>".parse().unwrap();
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
        let c: CustomHtmlTag = "<a key=\"val\"/>".parse().unwrap();
        assert_eq!(
            c,
            Inline(ComponentCall {
                name: "a".into(),
                attributes: BTreeMap::from([("key".into(), "val".into())])
            },)
        )
    }
}
