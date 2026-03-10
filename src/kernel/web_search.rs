//! Web search providers used by the tool executor.

use crate::error::{OraError, Result};
use regex::Regex;
use serde::Deserialize;
use std::sync::OnceLock;

const DEFAULT_BRAVE_BASE_URL: &str = "https://api.search.brave.com/res/v1/web/search";
const DEFAULT_DUCKDUCKGO_BASE_URL: &str = "https://html.duckduckgo.com/html/";
const DEFAULT_DUCKDUCKGO_INSTANT_URL: &str = "https://api.duckduckgo.com/";

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub source: String,
}

#[derive(Debug, Clone)]
enum SearchProvider {
    Brave { api_key: String, base_url: String },
    DuckDuckGo { html_base_url: String },
}

pub struct WebSearchService {
    client: reqwest::Client,
    provider: SearchProvider,
}

impl WebSearchService {
    pub fn from_env() -> Self {
        let client = reqwest::Client::builder()
            .user_agent(concat!("ora-rust/", env!("CARGO_PKG_VERSION")))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        let provider = match std::env::var("BRAVE_SEARCH_API_KEY") {
            Ok(api_key) if !api_key.trim().is_empty() => SearchProvider::Brave {
                api_key,
                base_url: std::env::var("ORA_WEB_SEARCH_BASE_URL")
                    .unwrap_or_else(|_| DEFAULT_BRAVE_BASE_URL.to_string()),
            },
            _ => SearchProvider::DuckDuckGo {
                html_base_url: std::env::var("ORA_WEB_SEARCH_BASE_URL")
                    .unwrap_or_else(|_| DEFAULT_DUCKDUCKGO_BASE_URL.to_string()),
            },
        };

        Self { client, provider }
    }

    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let limit = limit.clamp(1, 10);
        match &self.provider {
            SearchProvider::Brave { api_key, base_url } => {
                self.search_brave(query, limit, base_url, api_key).await
            }
            SearchProvider::DuckDuckGo { html_base_url } => {
                self.search_duckduckgo(query, limit, html_base_url).await
            }
        }
    }

    async fn search_brave(
        &self,
        query: &str,
        limit: usize,
        base_url: &str,
        api_key: &str,
    ) -> Result<Vec<SearchResult>> {
        let response = self
            .client
            .get(base_url)
            .header("Accept", "application/json")
            .header("X-Subscription-Token", api_key)
            .query(&[("q", query), ("count", &limit.to_string())])
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(OraError::NetworkError {
                message: format!(
                    "brave search request failed with status {}",
                    response.status()
                ),
            });
        }

        let payload: BraveSearchResponse = response.json().await?;
        let results = payload
            .web
            .map(|web| {
                web.results
                    .into_iter()
                    .take(limit)
                    .map(|result| SearchResult {
                        title: clean_html_fragment(&result.title),
                        url: result.url,
                        snippet: result
                            .description
                            .as_deref()
                            .map(clean_html_fragment)
                            .unwrap_or_default(),
                        source: "brave".to_string(),
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        Ok(results)
    }

    async fn search_duckduckgo(
        &self,
        query: &str,
        limit: usize,
        html_base_url: &str,
    ) -> Result<Vec<SearchResult>> {
        let response = self
            .client
            .get(html_base_url)
            .query(&[("q", query)])
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(OraError::NetworkError {
                message: format!(
                    "duckduckgo search request failed with status {}",
                    response.status()
                ),
            });
        }

        let html = response.text().await?;
        let mut results = parse_duckduckgo_html(&html, limit);
        if results.is_empty() {
            results = self.search_duckduckgo_instant(query, limit).await?;
        }

        Ok(results)
    }

    async fn search_duckduckgo_instant(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        let response = self
            .client
            .get(DEFAULT_DUCKDUCKGO_INSTANT_URL)
            .query(&[
                ("q", query),
                ("format", "json"),
                ("no_html", "1"),
                ("skip_disambig", "1"),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(OraError::NetworkError {
                message: format!(
                    "duckduckgo instant request failed with status {}",
                    response.status()
                ),
            });
        }

        let payload: DuckDuckGoInstantResponse = response.json().await?;
        let mut results = Vec::new();

        if !payload.abstract_text.trim().is_empty() && !payload.abstract_url.trim().is_empty() {
            results.push(SearchResult {
                title: if payload.heading.trim().is_empty() {
                    query.to_string()
                } else {
                    payload.heading.clone()
                },
                url: payload.abstract_url,
                snippet: payload.abstract_text,
                source: "duckduckgo".to_string(),
            });
        }

        for topic in flatten_related_topics(payload.related_topics)
            .into_iter()
            .take(limit.saturating_sub(results.len()))
        {
            if !topic.first_url.trim().is_empty() && !topic.text.trim().is_empty() {
                results.push(SearchResult {
                    title: topic.text.clone(),
                    url: topic.first_url,
                    snippet: topic.text,
                    source: "duckduckgo".to_string(),
                });
            }
        }

        Ok(results)
    }
}

#[derive(Debug, Deserialize)]
struct BraveSearchResponse {
    web: Option<BraveWebResults>,
}

#[derive(Debug, Deserialize)]
struct BraveWebResults {
    results: Vec<BraveResult>,
}

#[derive(Debug, Deserialize)]
struct BraveResult {
    title: String,
    url: String,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DuckDuckGoInstantResponse {
    #[serde(rename = "AbstractText")]
    abstract_text: String,
    #[serde(rename = "AbstractURL")]
    abstract_url: String,
    #[serde(rename = "Heading")]
    heading: String,
    #[serde(rename = "RelatedTopics")]
    related_topics: Vec<DuckDuckGoTopic>,
}

#[derive(Debug, Deserialize)]
struct DuckDuckGoTopic {
    #[serde(rename = "FirstURL")]
    first_url: String,
    #[serde(rename = "Text")]
    text: String,
    #[serde(rename = "Topics", default)]
    topics: Vec<DuckDuckGoTopic>,
}

fn flatten_related_topics(topics: Vec<DuckDuckGoTopic>) -> Vec<DuckDuckGoTopic> {
    let mut flattened = Vec::new();
    for topic in topics {
        if topic.topics.is_empty() {
            flattened.push(topic);
        } else {
            flattened.extend(flatten_related_topics(topic.topics));
        }
    }
    flattened
}

fn parse_duckduckgo_html(html: &str, limit: usize) -> Vec<SearchResult> {
    let snippets = snippet_regex()
        .captures_iter(html)
        .filter_map(|capture| capture.get(1).or_else(|| capture.get(2)))
        .map(|value| clean_html_fragment(value.as_str()))
        .collect::<Vec<_>>();

    link_regex()
        .captures_iter(html)
        .enumerate()
        .take(limit)
        .filter_map(|(index, capture)| {
            let href = capture.get(1)?.as_str();
            let title = clean_html_fragment(capture.get(2)?.as_str());
            let url = normalize_result_url(href);
            if title.is_empty() || url.is_empty() {
                return None;
            }

            Some(SearchResult {
                title,
                url,
                snippet: snippets.get(index).cloned().unwrap_or_default(),
                source: "duckduckgo".to_string(),
            })
        })
        .collect()
}

fn link_regex() -> &'static Regex {
    static LINK_REGEX: OnceLock<Regex> = OnceLock::new();
    LINK_REGEX.get_or_init(|| {
        Regex::new(r#"(?s)<a[^>]*class="result__a"[^>]*href="([^"]+)"[^>]*>(.*?)</a>"#)
            .expect("valid search result link regex")
    })
}

fn snippet_regex() -> &'static Regex {
    static SNIPPET_REGEX: OnceLock<Regex> = OnceLock::new();
    SNIPPET_REGEX.get_or_init(|| {
        Regex::new(
            r#"(?s)<a[^>]*class="result__snippet"[^>]*>(.*?)</a>|<div[^>]*class="result__snippet"[^>]*>(.*?)</div>"#,
        )
        .expect("valid search result snippet regex")
    })
}

fn normalize_result_url(raw_url: &str) -> String {
    let with_scheme = if raw_url.starts_with("//") {
        format!("https:{}", raw_url)
    } else {
        raw_url.to_string()
    };

    let Ok(parsed) = reqwest::Url::parse(&with_scheme) else {
        return decode_html_entities(&with_scheme);
    };

    if parsed.domain() == Some("duckduckgo.com") {
        if let Some((_, destination)) = parsed.query_pairs().find(|(key, _)| key == "uddg") {
            return destination.into_owned();
        }
    }

    parsed.to_string()
}

fn clean_html_fragment(fragment: &str) -> String {
    let decoded = decode_html_entities(fragment);
    let without_tags = tag_regex().replace_all(&decoded, " ");
    normalize_whitespace(&without_tags)
}

fn tag_regex() -> &'static Regex {
    static TAG_REGEX: OnceLock<Regex> = OnceLock::new();
    TAG_REGEX.get_or_init(|| Regex::new(r#"<[^>]+>"#).expect("valid html tag regex"))
}

fn normalize_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn decode_html_entities(text: &str) -> String {
    text.replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#x27;", "'")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&#x2F;", "/")
        .replace("&#47;", "/")
        .replace("&nbsp;", " ")
}

#[cfg(test)]
mod tests {
    use super::{normalize_result_url, parse_duckduckgo_html};

    #[test]
    fn parses_duckduckgo_html_results() {
        let html = r#"
            <div class="result">
                <a class="result__a" href="//duckduckgo.com/l/?uddg=https%3A%2F%2Fexample.com%2Fdocs">Example &amp; Docs</a>
                <a class="result__snippet">Fast &lt;strong&gt;reference&lt;/strong&gt; guide</a>
            </div>
            <div class="result">
                <a class="result__a" href="https://example.org/blog">Example Blog</a>
                <div class="result__snippet">Latest updates</div>
            </div>
        "#;

        let results = parse_duckduckgo_html(html, 10);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].title, "Example & Docs");
        assert_eq!(results[0].url, "https://example.com/docs");
        assert_eq!(results[0].snippet, "Fast reference guide");
        assert_eq!(results[1].url, "https://example.org/blog");
    }

    #[test]
    fn normalizes_duckduckgo_redirect_links() {
        let url = normalize_result_url(
            "//duckduckgo.com/l/?uddg=https%3A%2F%2Fexample.com%2Fhello%3Fq%3Done",
        );
        assert_eq!(url, "https://example.com/hello?q=one");
    }
}
