use pulldown_cmark_wikilink::{Tag, TagEnd};

pub fn as_closing_tag(t: &Tag) -> TagEnd {
    match t {
        Tag::Paragraph => TagEnd::Paragraph,
        Tag::Heading{level, ..} => TagEnd::Heading(*level),
        Tag::BlockQuote => TagEnd::BlockQuote,
        Tag::CodeBlock(_) => TagEnd::CodeBlock,
        Tag::List(b) => TagEnd::List(b.is_some()),
        Tag::Item => TagEnd::Item ,
        Tag::FootnoteDefinition(_) => TagEnd::FootnoteDefinition,
        Tag::Table(_) => TagEnd::Table,
        Tag::TableHead => TagEnd::TableHead ,
        Tag::TableRow => TagEnd::TableRow ,
        Tag::TableCell => TagEnd::TableCell ,
        Tag::Emphasis => TagEnd::Emphasis ,
        Tag::Strong => TagEnd::Strong ,
        Tag::Strikethrough => TagEnd::Strikethrough ,
        Tag::Link{..} => TagEnd::Link,
        Tag::Image{..} => TagEnd::Image,
        Tag::MetadataBlock(k) => TagEnd::MetadataBlock(*k),
        Tag::HtmlBlock => TagEnd::HtmlBlock,
    }
}

