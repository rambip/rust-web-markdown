use std::str::FromStr;
use core::iter::Peekable;

#[derive(Debug, PartialEq)]
// TODO: rename
pub struct ComponentCall {
    pub name: String, 
    pub attributes: Vec<(String, String)>,
}

// <a>
// content
// </a>
//
// probleme:
// <a>
//
//  inside
//
// <b>

#[derive(Debug, PartialEq)]
pub enum Stuff {
    Inline(ComponentCall),
    Start(ComponentCall),
    End(String),
}

// FIXME: better error handling
type ParseError = String;

fn parse_attribute_value(stream: &mut Peekable<std::str::Chars>) 
    -> Result<String, ParseError> {
    let mut attribute = String::new();

    if stream.next() != Some('"') {
        return Err("please use `\"` to wrap your attribute values".into())
    }

    loop {
        match stream.peek() {
            None => return Err("expected attribute value".into()),
            Some(&'"') => break,
            _ => attribute.push(stream.next().unwrap())
        }
    }
    stream.next();

    Ok(attribute)
}

fn parse_attribute_name(stream: &mut Peekable<std::str::Chars>) 
    -> Result<String, ParseError> {
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

fn parse_attribute(stream: &mut Peekable<std::str::Chars>) -> 
    Result<(String, String), ParseError> {
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

impl FromStr for Stuff {
    type Err = String;


    fn from_str(s: &str) -> Result<Stuff, Self::Err> {
        let mut stream = s.chars()
            .peekable();

        if stream.next() != Some('<') {
            return Err("expected <".into())
        }

        let is_end = if stream.peek() == Some(&'/') {
            stream.next();
            true
        }
        else {
            false
        };

        let mut name = String::new();

        loop {
            match stream.peek() {
                Some(&' ') | Some(&'/') | Some(&'>') => break,
                _ => name.push(stream.next().unwrap())
            }
        }

        let mut attributes = Vec::new();
        loop {
            match stream.peek() {
                None => return Err("expected end of tag".into()),
                Some(&'>') | Some(&'/') => break,
                _ => attributes.push(parse_attribute(&mut stream)?)
            }
        }

        if stream.peek() == Some(&'/') {
            return Ok(Stuff::Inline(ComponentCall {
                name,
                attributes,
            }))
        }

        while stream.peek() == Some(&' ') {
            stream.next();
        }

        if is_end {
            Ok(Stuff::End(name))
        }
        else {
            Ok(Stuff::Start(ComponentCall {
                name,
                attributes
            }))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use Stuff::*;

    #[test]
    fn parse_start(){
        let c : Stuff = "<a>".parse().unwrap();
        assert_eq!(c, Start(
                ComponentCall {
                    name: "a".into(),
                    attributes: [].into(),
                },
                )
        )
    }

    #[test]
    fn parse_end(){
        let c : Stuff = "</a>".parse().unwrap();
        assert_eq!(c, End("a".into()))
    }

    #[test]
    fn parse_inline_empty(){
        let c : Stuff = "<a/>".parse().unwrap();
        assert_eq!(c, Inline(
                ComponentCall {
                    name: "a".into(),
                    attributes: [].into(),
                },
                )
        )
    }

    #[test]
    fn parse_inline(){
        let c : Stuff = "<a key=\"val\"/>".parse().unwrap();
        assert_eq!(c, Inline(
                ComponentCall {
                    name: "a".into(),
                    attributes: vec![("key".into(), "val".into())]
                },
                )
        )
    }
}
