use std::str::FromStr;
use core::iter::Peekable;

#[derive(Debug)]
pub struct ComponentCall {
    pub name: String,
    pub attributes: Vec<(String, String)>,
    pub children: bool,
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

impl FromStr for ComponentCall {
    // FIXME: ParseError
    type Err = String;


    fn from_str(s: &str) -> Result<ComponentCall, Self::Err> {
        let mut stream = s.chars()
            .peekable();

        if stream.next() != Some('<') {
            return Err("expected <".into())
        }

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

        while stream.peek() == Some(&' ') {
            stream.next();
        }

        let empty_tag = stream.next()==Some('/') && stream.next()==Some('>');
        Ok(ComponentCall {
            name,
            attributes,
            children: !empty_tag
        })
    }
}

