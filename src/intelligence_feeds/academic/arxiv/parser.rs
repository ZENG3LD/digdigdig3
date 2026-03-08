//! arXiv response parsers
//!
//! Parse Atom XML responses to domain types.
//!
//! arXiv returns Atom XML format, not JSON. We parse it manually using
//! simple string operations to extract the key fields we need.

use crate::core::types::{ExchangeError, ExchangeResult};

pub struct ArxivParser;

impl ArxivParser {
    /// Parse arXiv Atom XML response to search result
    ///
    /// Example response structure:
    /// ```xml
    /// <?xml version="1.0" encoding="UTF-8"?>
    /// <feed xmlns="http://www.w3.org/2005/Atom">
    ///   <title>ArXiv Query...</title>
    ///   <opensearch:totalResults>100</opensearch:totalResults>
    ///   <opensearch:startIndex>0</opensearch:startIndex>
    ///   <entry>
    ///     <id>http://arxiv.org/abs/2301.12345v1</id>
    ///     <published>2023-01-15T12:00:00Z</published>
    ///     <updated>2023-01-15T12:00:00Z</updated>
    ///     <title>Paper Title</title>
    ///     <summary>Abstract text...</summary>
    ///     <author><name>Author Name</name></author>
    ///     <category term="cs.AI" scheme="http://arxiv.org/schemas/atom"/>
    ///     <link href="http://arxiv.org/abs/2301.12345v1" rel="alternate" type="text/html"/>
    ///     <link href="http://arxiv.org/pdf/2301.12345v1" rel="related" title="pdf" type="application/pdf"/>
    ///     <arxiv:doi xmlns:arxiv="http://arxiv.org/schemas/atom">10.1234/example</arxiv:doi>
    ///   </entry>
    /// </feed>
    /// ```
    pub fn parse_search_result(xml: &str) -> ExchangeResult<ArxivSearchResult> {
        // Extract total results
        let total_results = Self::extract_tag_content(xml, "opensearch:totalResults")
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

        // Extract start index
        let start_index = Self::extract_tag_content(xml, "opensearch:startIndex")
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

        // Parse each entry
        let mut papers = Vec::new();

        // Split by <entry> tags
        let entries: Vec<&str> = xml.split("<entry>").collect();

        for entry_text in entries.iter().skip(1) {
            // Find the closing tag
            if let Some(end_pos) = entry_text.find("</entry>") {
                let entry = &entry_text[..end_pos];

                if let Ok(paper) = Self::parse_entry(entry) {
                    papers.push(paper);
                }
            }
        }

        Ok(ArxivSearchResult {
            total_results,
            start_index,
            papers,
        })
    }

    /// Parse a single entry to ArxivPaper
    fn parse_entry(entry: &str) -> ExchangeResult<ArxivPaper> {
        // Extract ID (from <id>http://arxiv.org/abs/XXXX.XXXXX</id>)
        let id = Self::extract_tag_content(entry, "id")
            .and_then(|s| s.split("/abs/").last().map(|s| s.to_string()))
            .ok_or_else(|| ExchangeError::Parse("Missing paper ID".to_string()))?;

        // Extract title
        let title = Self::extract_tag_content(entry, "title")
            .ok_or_else(|| ExchangeError::Parse("Missing title".to_string()))?
            .trim()
            .replace('\n', " ")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");

        // Extract summary
        let summary = Self::extract_tag_content(entry, "summary")
            .ok_or_else(|| ExchangeError::Parse("Missing summary".to_string()))?
            .trim()
            .replace('\n', " ")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");

        // Extract authors
        let authors = Self::extract_all_authors(entry);

        // Extract published date
        let published = Self::extract_tag_content(entry, "published")
            .unwrap_or_default();

        // Extract updated date
        let updated = Self::extract_tag_content(entry, "updated")
            .unwrap_or_default();

        // Extract categories
        let categories = Self::extract_all_categories(entry);

        // Extract PDF URL
        let pdf_url = Self::extract_pdf_link(entry);

        // Extract DOI (optional)
        let doi = Self::extract_tag_content(entry, "arxiv:doi");

        Ok(ArxivPaper {
            id,
            title,
            summary,
            authors,
            published,
            updated,
            categories,
            pdf_url,
            doi,
        })
    }

    /// Extract content between opening and closing tags
    fn extract_tag_content(text: &str, tag: &str) -> Option<String> {
        let open_tag = format!("<{}>", tag);
        let close_tag = format!("</{}>", tag);

        let start = text.find(&open_tag)? + open_tag.len();
        let end = text[start..].find(&close_tag)? + start;

        Some(text[start..end].to_string())
    }

    /// Extract all author names
    fn extract_all_authors(entry: &str) -> Vec<String> {
        let mut authors = Vec::new();
        let mut search_start = 0;

        while let Some(pos) = entry[search_start..].find("<author>") {
            let author_start = search_start + pos;
            if let Some(author_end) = entry[author_start..].find("</author>") {
                let author_block = &entry[author_start..author_start + author_end];

                if let Some(name) = Self::extract_tag_content(author_block, "name") {
                    authors.push(name.trim().to_string());
                }

                search_start = author_start + author_end + "</author>".len();
            } else {
                break;
            }
        }

        authors
    }

    /// Extract all category terms
    fn extract_all_categories(entry: &str) -> Vec<String> {
        let mut categories = Vec::new();
        let mut search_start = 0;

        while let Some(pos) = entry[search_start..].find("<category ") {
            let cat_start = search_start + pos;
            if let Some(cat_end) = entry[cat_start..].find("/>") {
                let cat_tag = &entry[cat_start..cat_start + cat_end];

                // Extract term attribute: term="cs.AI"
                if let Some(term_start) = cat_tag.find("term=\"") {
                    let term_value_start = term_start + "term=\"".len();
                    if let Some(term_end) = cat_tag[term_value_start..].find('"') {
                        let term = cat_tag[term_value_start..term_value_start + term_end].to_string();
                        categories.push(term);
                    }
                }

                search_start = cat_start + cat_end + "/>".len();
            } else {
                break;
            }
        }

        categories
    }

    /// Extract PDF link
    fn extract_pdf_link(entry: &str) -> Option<String> {
        let mut search_start = 0;

        while let Some(pos) = entry[search_start..].find("<link ") {
            let link_start = search_start + pos;
            if let Some(link_end) = entry[link_start..].find("/>") {
                let link_tag = &entry[link_start..link_start + link_end];

                // Check if this is a PDF link: title="pdf"
                if link_tag.contains("title=\"pdf\"") || link_tag.contains("type=\"application/pdf\"") {
                    // Extract href attribute
                    if let Some(href_start) = link_tag.find("href=\"") {
                        let href_value_start = href_start + "href=\"".len();
                        if let Some(href_end) = link_tag[href_value_start..].find('"') {
                            return Some(link_tag[href_value_start..href_value_start + href_end].to_string());
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
}

// =============================================================================
// arXiv-SPECIFIC TYPES
// =============================================================================

/// arXiv paper metadata
#[derive(Debug, Clone)]
pub struct ArxivPaper {
    /// arXiv ID (e.g., "2301.12345v1")
    pub id: String,
    /// Paper title
    pub title: String,
    /// Abstract/summary
    pub summary: String,
    /// List of author names
    pub authors: Vec<String>,
    /// Publication date (ISO 8601)
    pub published: String,
    /// Last updated date (ISO 8601)
    pub updated: String,
    /// Category tags (e.g., "cs.AI", "q-fin.TR")
    pub categories: Vec<String>,
    /// PDF download URL
    pub pdf_url: Option<String>,
    /// DOI (Digital Object Identifier)
    pub doi: Option<String>,
}

/// arXiv search result
#[derive(Debug, Clone)]
pub struct ArxivSearchResult {
    /// Total number of results
    pub total_results: u64,
    /// Start index of this page
    pub start_index: u64,
    /// Papers in this result
    pub papers: Vec<ArxivPaper>,
}
