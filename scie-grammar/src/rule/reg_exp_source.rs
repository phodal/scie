use crate::rule::CompiledRule;
use regex::{Captures, Regex};
use scie_scanner::scanner::scie_scanner::IOnigCaptureIndex;

lazy_static! {
    static ref HAS_BACK_REFERENCES: Regex = Regex::new(r"\\(\d+)").unwrap();
    static ref BACK_REFERENCING_END: Regex = Regex::new(r"\\(\d+)").unwrap();
    static ref REG_EXP_REGEX: Regex = Regex::new(r"[\-\\\{\}\*\+\?\|\^\$.,\[\]\(\)\#\s]").unwrap();
}

#[derive(Clone, Debug, Serialize)]
pub struct IRegExpSourceListAnchorCache {
    a0_g0: Option<Box<CompiledRule>>,
    a0_g1: Option<Box<CompiledRule>>,
    a1_g0: Option<Box<CompiledRule>>,
    a1_g1: Option<Box<CompiledRule>>,
}

impl Default for IRegExpSourceListAnchorCache {
    fn default() -> Self {
        IRegExpSourceListAnchorCache {
            a0_g0: None,
            a0_g1: None,
            a1_g0: None,
            a1_g1: None,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct AnchorCache {
    a0_g0: String,
    a0_g1: String,
    a1_g0: String,
    a1_g1: String,
}

impl Default for AnchorCache {
    fn default() -> Self {
        AnchorCache {
            a0_g0: String::from(""),
            a0_g1: String::from(""),
            a1_g0: String::from(""),
            a1_g1: String::from(""),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct RegExpSourceList {
    pub _has_anchors: bool,
    pub _cached: Option<CompiledRule>,
    pub _anchor_cache: IRegExpSourceListAnchorCache,
    pub _items: Vec<RegExpSource>,
}

impl RegExpSourceList {
    pub fn new() -> Self {
        RegExpSourceList {
            _has_anchors: false,
            _cached: None,
            _anchor_cache: Default::default(),
            _items: vec![],
        }
    }

    pub fn push(&mut self, item: RegExpSource) {
        let item_has_anchor = item.has_anchor.clone();
        self._items.push(item);

        let has_anchor = self._has_anchors || item_has_anchor;
        self._has_anchors = has_anchor;
    }

    pub fn unshift(&mut self, item: RegExpSource) {
        let item_has_anchor = item.has_anchor.clone();

        self._items.push(item);
        self._items.rotate_right(1);

        self._has_anchors = self._has_anchors || item_has_anchor;
    }

    pub fn compile(&mut self, allow_a: bool, allow_g: bool) -> Box<CompiledRule> {
        if !self._has_anchors {
            if self._cached.is_none() {
                let mut reg_exps = vec![];
                let mut rules = vec![];
                for x in self._items.iter() {
                    reg_exps.push(x.resolve_anchors(allow_a, allow_g));
                    rules.push(x.rule_id);
                }

                self._cached = Some(CompiledRule::new(reg_exps, rules));
            }

            return Box::from(self._cached.clone().unwrap());
        } else {
            if allow_a {
                if allow_g {
                    if self._anchor_cache.a1_g1.is_none() {
                        self._anchor_cache.a1_g1 = Some(self.resolve_anchors(allow_a, allow_g));
                    }
                    return self._anchor_cache.a1_g1.clone().unwrap();
                } else {
                    if self._anchor_cache.a1_g0.is_none() {
                        self._anchor_cache.a1_g0 = Some(self.resolve_anchors(allow_a, allow_g));
                    }
                    return self._anchor_cache.a1_g0.clone().unwrap();
                }
            } else {
                if allow_g {
                    if self._anchor_cache.a0_g1.is_none() {
                        self._anchor_cache.a0_g1 = Some(self.resolve_anchors(allow_a, allow_g));
                    }
                    return self._anchor_cache.a0_g1.clone().unwrap();
                } else {
                    if self._anchor_cache.a0_g0.is_none() {
                        self._anchor_cache.a0_g0 = Some(self.resolve_anchors(allow_a, allow_g));
                    }
                    return self._anchor_cache.a0_g0.clone().unwrap();
                }
            }
        }
    }

    fn resolve_anchors(&self, allow_a: bool, allow_g: bool) -> Box<CompiledRule> {
        let mut reg_exps = vec![];
        let mut rules = vec![];
        for x in self._items.iter() {
            reg_exps.push(x.resolve_anchors(allow_a, allow_g));
            rules.push(x.rule_id);
        }

        Box::from(CompiledRule::new(reg_exps, rules))
    }

    pub fn length(&self) -> usize {
        return self._items.len();
    }

    pub fn dispose_caches(&mut self) {
        if self._cached.is_some() {
            self._cached = None;
        }
        if self._anchor_cache.a0_g0.is_some() {
            self._anchor_cache.a0_g0 = None;
        }
        if self._anchor_cache.a0_g1.is_some() {
            self._anchor_cache.a0_g1 = None;
        }
        if self._anchor_cache.a1_g0.is_some() {
            self._anchor_cache.a1_g0 = None;
        }
        if self._anchor_cache.a1_g1.is_some() {
            self._anchor_cache.a1_g1 = None;
        }
    }

    pub fn set_source(&mut self, index: usize, new_source: &str) {
        if self._items[index].source != new_source {
            self.dispose_caches();
            self._items[index].set_source(new_source);
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct RegExpSource {
    pub source: String,
    pub rule_id: i32,
    pub has_anchor: bool,
    _anchor_cache: Option<AnchorCache>,
    pub has_back_references: bool,
}

impl RegExpSource {
    pub fn new(exp_source: String, rule_id: i32) -> RegExpSource {
        let mut has_anchor = false;

        let result: String;
        let length = exp_source.len();
        let mut output: Vec<&str> = vec![];
        let mut last_pushed_pos = 0;

        let mut pos = 0;
        let chars: Vec<char> = exp_source.chars().collect();

        while pos < length {
            let ch = chars[pos];
            if ch == '\\' {
                if pos + 1 < length {
                    let next_char = chars[pos + 1];
                    if next_char == 'z' {
                        output.push(&exp_source[last_pushed_pos..pos]);
                        output.push("$(?!\n)(?<!\n)");
                        last_pushed_pos = pos + 2;
                    } else if next_char == 'G' || next_char == 'A' {
                        has_anchor = true
                    }

                    pos = pos + 1;
                }
            }

            pos = pos + 1;
        }

        let anchor_cache: Option<AnchorCache> = None;

        if last_pushed_pos == 0 {
            result = exp_source.clone()
        } else {
            output.push(&exp_source[last_pushed_pos..length]);
            result = output.join("");
        }

        let mut reg_exp_source = RegExpSource {
            source: result,
            rule_id,
            has_anchor,
            _anchor_cache: anchor_cache,
            has_back_references: false,
        };

        reg_exp_source._anchor_cache = Some(reg_exp_source.build_anchor_cache());

        if HAS_BACK_REFERENCES.is_match(reg_exp_source.source.as_str()) {
            reg_exp_source.has_back_references = true;
        }

        reg_exp_source
    }

    fn build_anchor_cache(&self) -> AnchorCache {
        let length = self.source.len();

        let mut a0_g0_result: Vec<String> = vec![];
        let mut a0_g1_result: Vec<String> = vec![];
        let mut a1_g0_result: Vec<String> = vec![];
        let mut a1_g1_result: Vec<String> = vec![];

        a0_g0_result.resize(length, String::from(""));
        a0_g1_result.resize(length, String::from(""));
        a1_g0_result.resize(length, String::from(""));
        a1_g1_result.resize(length, String::from(""));

        let mut pos = 0;
        let mut ch: char;
        let mut next_char: char;
        let chars: Vec<char> = self.source.chars().collect();

        while pos < length {
            ch = chars[pos];
            a0_g0_result[pos] = ch.to_string();
            a0_g1_result[pos] = ch.to_string();
            a1_g0_result[pos] = ch.to_string();
            a1_g1_result[pos] = ch.to_string();

            if ch == '\\' {
                if pos + 1 < length {
                    next_char = chars[pos + 1];
                    if next_char == 'A' {
                        a0_g0_result[pos + 1] = String::from("\u{FFFF}");
                        a0_g1_result[pos + 1] = String::from("\u{FFFF}");
                        a1_g0_result[pos + 1] = String::from("A");
                        a1_g1_result[pos + 1] = String::from("A");
                    } else if next_char == 'G' {
                        a0_g0_result[pos + 1] = String::from("\u{FFFF}");
                        a0_g1_result[pos + 1] = String::from("G");
                        a1_g0_result[pos + 1] = String::from("\u{FFFF}");
                        a1_g1_result[pos + 1] = String::from("G");
                    } else {
                        a0_g0_result[pos + 1] = String::from(next_char.clone());
                        a0_g1_result[pos + 1] = String::from(next_char.clone());
                        a1_g0_result[pos + 1] = String::from(next_char.clone());
                        a1_g1_result[pos + 1] = String::from(next_char.clone());
                    }

                    pos = pos + 1;
                }
            }

            pos = pos + 1;
        }

        return AnchorCache {
            a0_g0: a0_g0_result.join(""),
            a0_g1: a0_g1_result.join(""),
            a1_g0: a1_g0_result.join(""),
            a1_g1: a1_g1_result.join(""),
        };
    }

    fn resolve_anchors(&self, allow_a: bool, allow_g: bool) -> String {
        if !self.has_anchor || self._anchor_cache.is_none() {
            return self.source.clone();
        }

        let cached = self._anchor_cache.as_ref().unwrap();
        if allow_a {
            if allow_g {
                return cached.a1_g1.clone();
            } else {
                return cached.a1_g0.clone();
            }
        } else {
            if allow_g {
                return cached.a0_g1.clone();
            } else {
                return cached.a0_g0.clone();
            }
        }
    }

    fn set_source(&mut self, new_source: &str) {
        if self.source == new_source {
            return;
        }

        self.source = String::from(new_source);

        if self.has_anchor {
            self._anchor_cache = Some(self.build_anchor_cache());
        }
    }

    pub fn resolve_back_references(
        &self,
        line_text: &str,
        capture_indices: Vec<IOnigCaptureIndex>,
    ) -> String {
        let captured_values: Vec<String> = capture_indices
            .into_iter()
            .map(|x| {
                return line_text[x.start..x.end].to_string();
            })
            .collect();

        let result = BACK_REFERENCING_END
            .replace(&*self.source, |caps: &Captures| {
                let index = caps.get(1).unwrap().as_str().parse::<usize>().unwrap();
                let mut chars = "";
                if captured_values.get(index).is_some() {
                    chars = &*captured_values[index];
                };

                return REG_EXP_REGEX.replace(&chars, "\\$&").to_string();
            })
            .to_string();

        return result;
    }
}

#[cfg(test)]
mod tests {
    use crate::rule::RegExpSource;

    #[test]
    fn should_change_resource_for_g() {
        let source = RegExpSource::new(String::from("\\G"), 1);
        assert!(source.has_anchor);
    }

    #[test]
    fn should_change_resource_for_z() {
        let source = RegExpSource::new(String::from("\\z"), 1);
        assert_eq!("$(?!\n)(?<!\n)", source.source);
    }

    #[test]
    fn should_build_anchor_cache_for_g() {
        let source = RegExpSource::new(String::from("\\G"), 1);
        let cache = source._anchor_cache.unwrap();
        assert_eq!("\\\u{ffff}", cache.a0_g0);
        assert_eq!("\\G", cache.a0_g1);
        assert_eq!("\\\u{ffff}", cache.a1_g0);
        assert_eq!("\\G", cache.a1_g1);
    }

    #[test]
    fn should_build_anchor_cache_for_g_source() {
        let source = RegExpSource::new(String::from("\\G(?!\n)"), 1);
        let cache = source._anchor_cache.unwrap();
        assert_eq!("\\\u{ffff}(?!\n)", cache.a0_g0);
        assert_eq!("\\G(?!\n)", cache.a0_g1);
        assert_eq!("\\\u{ffff}(?!\n)", cache.a1_g0);
        assert_eq!("\\G(?!\n)", cache.a1_g1);
    }

    #[test]
    fn should_build_anchor_cache_for_long() {
        let source = RegExpSource::new(
            String::from("(^[ ]*|\\G\\s*)([^\\s]+)\\s*(=|\\?=|:=|\\+=)"),
            1,
        );
        let cache = source._anchor_cache.unwrap();
        assert_eq!(
            "(^[ ]*|\\\u{ffff}\\s*)([^\\s]+)\\s*(=|\\?=|:=|\\+=)",
            cache.a0_g0
        );
        assert_eq!("(^[ ]*|\\G\\s*)([^\\s]+)\\s*(=|\\?=|:=|\\+=)", cache.a0_g1);
        assert_eq!("(^[ ]*|\\￿\\s*)([^\\s]+)\\s*(=|\\?=|:=|\\+=)", cache.a1_g0);
        assert_eq!("(^[ ]*|\\G\\s*)([^\\s]+)\\s*(=|\\?=|:=|\\+=)", cache.a1_g1);
    }

    #[test]
    fn should_return_true_when_has_back_refs() {
        let source = RegExpSource::new(String::from("(>(<)/)(\\2)(>)"), 1);
        assert_eq!(true, source.has_back_references);
    }
}
