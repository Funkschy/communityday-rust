use html5ever::{interface::Attribute, tendril::*, tokenizer::*};
use url::{ParseError, Url};

#[derive(Copy, Clone)]
pub struct LinkStr(usize);

#[derive(Default)]
pub struct LinkFinder {
    base: String,
    links: Vec<StrTendril>,
    pub link_strings: Vec<LinkStr>,
}

impl LinkFinder {
    pub fn get_url(&self, link_str: LinkStr) -> Option<Url> {
        let url = self.links.get(link_str.0)?;
        let parse_res = Url::parse(url);

        match parse_res {
            Ok(url) => Some(url),
            Err(ParseError::RelativeUrlWithoutBase) => {
                let base = Url::parse(&self.base).ok()?;
                if !Self::is_loop(&base, &**url) {
                    base.join(url).ok()
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }

    fn is_loop(base: &Url, link: &str) -> bool {
        return base.path().contains(link);
    }

    fn push_href(&mut self, attrs: Vec<Attribute>) {
        for attr in attrs {
            if attr.name.local.eq("href") {
                self.links.push(attr.value);
                // an <a> tag should only have one href
                break;
            }
        }
    }
}

impl LinkFinder {
    pub fn get_links(base: String, html: &str) -> Self {
        let mut bufq = Self::get_buffer_queue(html);
        let mut link_finder = {
            let link_finder = Self::default();
            let mut tokenizer: Tokenizer<Self> = link_finder.into();

            let _ = tokenizer.feed(&mut bufq);
            tokenizer.end();
            tokenizer.sink
        };

        link_finder.link_strings = (0_usize..link_finder.links.len())
            .into_iter()
            .map(|i| LinkStr(i))
            .collect();

        link_finder.base = base;
        link_finder
    }

    fn get_buffer_queue(html: &str) -> BufferQueue {
        let tendril = StrTendril::from(html);
        let mut queue = BufferQueue::new();
        queue.push_back(tendril);
        queue
    }
}

impl Into<Tokenizer<LinkFinder>> for LinkFinder {
    fn into(self) -> Tokenizer<LinkFinder> {
        Tokenizer::new(self, TokenizerOpts::default())
    }
}

impl TokenSink for LinkFinder {
    type Handle = ();

    fn process_token(&mut self, token: Token, _line_number: u64) -> TokenSinkResult<()> {
        if let TagToken(tag) = token {
            if tag.kind == StartTag && tag.name.eq("a") {
                self.push_href(tag.attrs);
            }
        }

        TokenSinkResult::Continue
    }
}
