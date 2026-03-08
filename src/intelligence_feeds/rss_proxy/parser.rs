//! RSS Feed Proxy response parsers
//!
//! Parse RSS 2.0 and Atom XML feeds to domain types.
//!
//! Supports both RSS 2.0 and Atom 1.0 formats.

use crate::core::types::{ExchangeError, ExchangeResult};

pub struct RssProxyParser;

impl RssProxyParser {
    /// Parse RSS or Atom feed (auto-detect format)
    pub fn parse_feed(xml: &str) -> ExchangeResult<RssFeed> {
        // Auto-detect format based on root element
        if xml.contains("<feed") && xml.contains("xmlns=\"http://www.w3.org/2005/Atom\"") {
            Self::parse_atom_feed(xml)
        } else if xml.contains("<rss") || xml.contains("<channel>") {
            Self::parse_rss_feed(xml)
        } else {
            Err(ExchangeError::Parse(
                "Unknown feed format (not RSS or Atom)".to_string(),
            ))
        }
    }

    /// Parse RSS 2.0 feed
    ///
    /// Example RSS structure:
    /// ```xml
    /// <?xml version="1.0"?>
    /// <rss version="2.0">
    ///   <channel>
    ///     <title>Feed Title</title>
    ///     <link>https://example.com</link>
    ///     <description>Feed description</description>
    ///     <language>en-us</language>
    ///     <lastBuildDate>Mon, 30 Dec 2024 12:00:00 GMT</lastBuildDate>
    ///     <item>
    ///       <title>Item Title</title>
    ///       <link>https://example.com/article</link>
    ///       <description>Item description</description>
    ///       <pubDate>Mon, 30 Dec 2024 12:00:00 GMT</pubDate>
    ///       <author>author@example.com</author>
    ///       <category>Technology</category>
    ///       <guid>https://example.com/article</guid>
    ///     </item>
    ///   </channel>
    /// </rss>
    /// ```
    pub fn parse_rss_feed(xml: &str) -> ExchangeResult<RssFeed> {
        // Extract channel section
        let channel = Self::extract_section(xml, "channel")
            .ok_or_else(|| ExchangeError::Parse("No <channel> found in RSS".to_string()))?;

        // Extract feed metadata
        let title = Self::extract_tag_content(&channel, "title").unwrap_or_default();
        let link = Self::extract_tag_content(&channel, "link").unwrap_or_default();
        let description = Self::extract_tag_content(&channel, "description").unwrap_or_default();
        let language = Self::extract_tag_content(&channel, "language");
        let last_build_date = Self::extract_tag_content(&channel, "lastBuildDate");

        // Parse items
        let mut items = Vec::new();
        let mut search_start = 0;

        while let Some(pos) = channel[search_start..].find("<item>") {
            let item_start = search_start + pos;
            if let Some(item_end_pos) = channel[item_start..].find("</item>") {
                let item_end = item_start + item_end_pos;
                let item_xml = &channel[item_start..item_end];

                if let Ok(item) = Self::parse_rss_item(item_xml) {
                    items.push(item);
                }

                search_start = item_end + "</item>".len();
            } else {
                break;
            }
        }

        Ok(RssFeed {
            title,
            link,
            description,
            language,
            items,
            last_build_date,
        })
    }

    /// Parse Atom 1.0 feed
    ///
    /// Example Atom structure:
    /// ```xml
    /// <?xml version="1.0"?>
    /// <feed xmlns="http://www.w3.org/2005/Atom">
    ///   <title>Feed Title</title>
    ///   <link href="https://example.com"/>
    ///   <subtitle>Feed description</subtitle>
    ///   <updated>2024-12-30T12:00:00Z</updated>
    ///   <entry>
    ///     <title>Entry Title</title>
    ///     <link href="https://example.com/article"/>
    ///     <summary>Entry summary</summary>
    ///     <published>2024-12-30T12:00:00Z</published>
    ///     <author><name>Author Name</name></author>
    ///     <category term="technology"/>
    ///     <id>https://example.com/article</id>
    ///   </entry>
    /// </feed>
    /// ```
    pub fn parse_atom_feed(xml: &str) -> ExchangeResult<RssFeed> {
        // Extract feed metadata
        let title = Self::extract_tag_content(xml, "title").unwrap_or_default();
        let link = Self::extract_atom_link(xml).unwrap_or_default();
        let description = Self::extract_tag_content(xml, "subtitle")
            .or_else(|| Self::extract_tag_content(xml, "summary"))
            .unwrap_or_default();
        let language = None; // Atom doesn't have a standard language field
        let last_build_date = Self::extract_tag_content(xml, "updated");

        // Parse entries
        let mut items = Vec::new();
        let mut search_start = 0;

        while let Some(pos) = xml[search_start..].find("<entry>") {
            let entry_start = search_start + pos;
            if let Some(entry_end_pos) = xml[entry_start..].find("</entry>") {
                let entry_end = entry_start + entry_end_pos;
                let entry_xml = &xml[entry_start..entry_end];

                if let Ok(item) = Self::parse_atom_entry(entry_xml) {
                    items.push(item);
                }

                search_start = entry_end + "</entry>".len();
            } else {
                break;
            }
        }

        Ok(RssFeed {
            title,
            link,
            description,
            language,
            items,
            last_build_date,
        })
    }

    /// Parse a single RSS <item>
    fn parse_rss_item(xml: &str) -> ExchangeResult<RssFeedItem> {
        let title = Self::extract_tag_content(xml, "title")
            .ok_or_else(|| ExchangeError::Parse("Missing item title".to_string()))?
            .trim()
            .to_string();

        let link = Self::extract_tag_content(xml, "link")
            .unwrap_or_default()
            .trim()
            .to_string();

        let description = Self::extract_tag_content(xml, "description")
            .or_else(|| Self::extract_tag_content(xml, "content:encoded"))
            .unwrap_or_default()
            .trim()
            .to_string();

        let pub_date = Self::extract_tag_content(xml, "pubDate");

        let author = Self::extract_tag_content(xml, "author")
            .or_else(|| Self::extract_tag_content(xml, "dc:creator"));

        let categories = Self::extract_all_tag_contents(xml, "category");

        let guid = Self::extract_tag_content(xml, "guid")
            .or_else(|| Self::extract_tag_content(xml, "link"));

        // Extract source name if present
        let source_name = Self::extract_tag_content(xml, "source");

        Ok(RssFeedItem {
            title,
            link,
            description,
            pub_date,
            author,
            categories,
            guid,
            source_name,
        })
    }

    /// Parse a single Atom <entry>
    fn parse_atom_entry(xml: &str) -> ExchangeResult<RssFeedItem> {
        let title = Self::extract_tag_content(xml, "title")
            .ok_or_else(|| ExchangeError::Parse("Missing entry title".to_string()))?
            .trim()
            .to_string();

        let link = Self::extract_atom_link(xml).unwrap_or_default();

        let description = Self::extract_tag_content(xml, "summary")
            .or_else(|| Self::extract_tag_content(xml, "content"))
            .unwrap_or_default()
            .trim()
            .to_string();

        let pub_date = Self::extract_tag_content(xml, "published")
            .or_else(|| Self::extract_tag_content(xml, "updated"));

        let author = Self::extract_atom_author(xml);

        let categories = Self::extract_atom_categories(xml);

        let guid = Self::extract_tag_content(xml, "id");

        // Atom doesn't have source in entries typically
        let source_name = None;

        Ok(RssFeedItem {
            title,
            link,
            description,
            pub_date,
            author,
            categories,
            guid,
            source_name,
        })
    }

    // ==========================================================================
    // HELPER METHODS
    // ==========================================================================

    /// Extract content between opening and closing tags
    fn extract_tag_content(text: &str, tag: &str) -> Option<String> {
        let open_tag = format!("<{}>", tag);
        let close_tag = format!("</{}>", tag);

        let start = text.find(&open_tag)? + open_tag.len();
        let end = text[start..].find(&close_tag)? + start;

        Some(Self::decode_html_entities(&text[start..end]))
    }

    /// Extract all occurrences of a tag
    fn extract_all_tag_contents(text: &str, tag: &str) -> Vec<String> {
        let mut results = Vec::new();
        let open_tag = format!("<{}>", tag);
        let close_tag = format!("</{}>", tag);
        let mut search_start = 0;

        while let Some(pos) = text[search_start..].find(&open_tag) {
            let start = search_start + pos + open_tag.len();
            if let Some(end_pos) = text[start..].find(&close_tag) {
                let end = start + end_pos;
                results.push(Self::decode_html_entities(&text[start..end]));
                search_start = end + close_tag.len();
            } else {
                break;
            }
        }

        results
    }

    /// Extract section between tags (including nested tags)
    fn extract_section(text: &str, tag: &str) -> Option<String> {
        let open_tag = format!("<{}", tag);
        let close_tag = format!("</{}>", tag);

        let start_pos = text.find(&open_tag)?;
        // Find the end of the opening tag
        let content_start = text[start_pos..].find('>')? + start_pos + 1;
        let end = text[content_start..].find(&close_tag)? + content_start;

        Some(text[content_start..end].to_string())
    }

    /// Extract Atom link (from <link href="..."/>)
    fn extract_atom_link(text: &str) -> Option<String> {
        let mut search_start = 0;

        while let Some(pos) = text[search_start..].find("<link ") {
            let link_start = search_start + pos;
            if let Some(link_end) = text[link_start..].find("/>") {
                let link_tag = &text[link_start..link_start + link_end];

                // Look for rel="alternate" or no rel attribute
                if !link_tag.contains("rel=") || link_tag.contains("rel=\"alternate\"") {
                    // Extract href
                    if let Some(href_start) = link_tag.find("href=\"") {
                        let href_value_start = href_start + "href=\"".len();
                        if let Some(href_end) = link_tag[href_value_start..].find('"') {
                            return Some(
                                link_tag[href_value_start..href_value_start + href_end].to_string(),
                            );
                        }
                    }
                }

                search_start = link_start + link_end + "/>".len();
            } else {
                break;
            }
        }

        None
    }

    /// Extract Atom author name
    fn extract_atom_author(text: &str) -> Option<String> {
        // Find <author> block
        if let Some(author_start) = text.find("<author>") {
            if let Some(author_end) = text[author_start..].find("</author>") {
                let author_block = &text[author_start..author_start + author_end];
                return Self::extract_tag_content(author_block, "name");
            }
        }
        None
    }

    /// Extract Atom categories
    fn extract_atom_categories(text: &str) -> Vec<String> {
        let mut categories = Vec::new();
        let mut search_start = 0;

        while let Some(pos) = text[search_start..].find("<category ") {
            let cat_start = search_start + pos;
            if let Some(cat_end) = text[cat_start..].find("/>") {
                let cat_tag = &text[cat_start..cat_start + cat_end];

                // Extract term attribute: term="technology"
                if let Some(term_start) = cat_tag.find("term=\"") {
                    let term_value_start = term_start + "term=\"".len();
                    if let Some(term_end) = cat_tag[term_value_start..].find('"') {
                        categories
                            .push(cat_tag[term_value_start..term_value_start + term_end].to_string());
                    }
                }

                search_start = cat_start + cat_end + "/>".len();
            } else {
                break;
            }
        }

        categories
    }

    /// Decode common HTML entities
    fn decode_html_entities(text: &str) -> String {
        text.replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&apos;", "'")
            .replace("&#39;", "'")
    }
}

// =============================================================================
// RSS TYPES
// =============================================================================

/// RSS feed item
#[derive(Debug, Clone)]
pub struct RssFeedItem {
    /// Item title
    pub title: String,
    /// Item link/URL
    pub link: String,
    /// Item description/content
    pub description: String,
    /// Publication date
    pub pub_date: Option<String>,
    /// Author name or email
    pub author: Option<String>,
    /// Category tags
    pub categories: Vec<String>,
    /// Globally unique identifier
    pub guid: Option<String>,
    /// Source feed name (if aggregated)
    pub source_name: Option<String>,
}

/// RSS feed
#[derive(Debug, Clone)]
pub struct RssFeed {
    /// Feed title
    pub title: String,
    /// Feed link/URL
    pub link: String,
    /// Feed description
    pub description: String,
    /// Feed language code (e.g., "en-us")
    pub language: Option<String>,
    /// Feed items
    pub items: Vec<RssFeedItem>,
    /// Last build/update date
    pub last_build_date: Option<String>,
}

impl RssFeed {
    /// Get the source name from the feed (for aggregation)
    pub fn source_name(&self) -> &str {
        &self.title
    }

    /// Get number of items
    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    /// Get items sorted by date (newest first)
    pub fn items_by_date(&self) -> Vec<&RssFeedItem> {
        let mut sorted = self.items.iter().collect::<Vec<_>>();
        sorted.sort_by(|a, b| b.pub_date.cmp(&a.pub_date));
        sorted
    }
}
