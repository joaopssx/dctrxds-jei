//! A substring index: which ingredients have a text containing a word.
//!
//! JEI's storages (`LuceneSearchStorage`, `LimitedStringStorage`) answer the
//! same question through Lucene wildcards. Here each distinct text is kept
//! once, one after the other in a single string, so a search is one scan of
//! that string, however many ingredients share a text.

use std::collections::HashMap;

/// A set of ingredient numbers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bits {
    words: Vec<u64>,
    len: usize,
}

impl Bits {
    pub fn empty(len: usize) -> Self {
        Self {
            words: vec![0; len.div_ceil(64)],
            len,
        }
    }

    pub fn full(len: usize) -> Self {
        let mut bits = Self {
            words: vec![u64::MAX; len.div_ceil(64)],
            len,
        };
        if !len.is_multiple_of(64)
            && let Some(last) = bits.words.last_mut()
        {
            *last = (1 << (len % 64)) - 1;
        }
        bits
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.words.iter().all(|&w| w == 0)
    }

    pub fn count(&self) -> usize {
        self.words.iter().map(|w| w.count_ones() as usize).sum()
    }

    pub fn set(&mut self, n: usize) {
        if n < self.len {
            self.words[n / 64] |= 1 << (n % 64);
        }
    }

    pub fn clear(&mut self, n: usize) {
        if n < self.len {
            self.words[n / 64] &= !(1 << (n % 64));
        }
    }

    pub fn contains(&self, n: usize) -> bool {
        n < self.len && self.words[n / 64] & (1 << (n % 64)) != 0
    }

    pub fn union_with(&mut self, other: &Self) {
        for (a, b) in self.words.iter_mut().zip(&other.words) {
            *a |= b;
        }
    }

    pub fn intersect_with(&mut self, other: &Self) {
        for (a, b) in self.words.iter_mut().zip(&other.words) {
            *a &= b;
        }
    }

    pub fn subtract(&mut self, other: &Self) {
        for (a, b) in self.words.iter_mut().zip(&other.words) {
            *a &= !b;
        }
    }
}

/// Collects `(text, ingredient)` pairs for a [`TextIndex`].
#[derive(Debug, Default)]
pub struct TextIndexBuilder {
    groups: HashMap<String, Vec<u32>>,
}

impl TextIndexBuilder {
    /// Keeps a text for an ingredient; blank texts are left out, and the text
    /// is trimmed, as JEI's `ElementSearch.putIfNotBlank` does.
    pub fn put(&mut self, text: &str, ingredient: u32) {
        let text = text.trim();
        if !text.is_empty() {
            self.groups
                .entry(text.to_owned())
                .or_default()
                .push(ingredient);
        }
    }

    pub fn build(self) -> TextIndex {
        let mut groups: Vec<(String, Vec<u32>)> = self.groups.into_iter().collect();
        groups.sort_by(|a, b| a.0.cmp(&b.0));
        let mut text = String::new();
        let mut starts = Vec::with_capacity(groups.len());
        let mut members = Vec::with_capacity(groups.len());
        for (key, ingredients) in groups {
            starts.push(text.len());
            text.push_str(&key);
            // Never part of a search word: a match can't run across two texts.
            text.push('\0');
            members.push(ingredients);
        }
        TextIndex {
            text,
            starts,
            members,
        }
    }
}

/// Each distinct text once, and the ingredients that have it.
#[derive(Debug, Default)]
pub struct TextIndex {
    text: String,
    starts: Vec<usize>,
    members: Vec<Vec<u32>>,
}

impl TextIndex {
    /// How many distinct texts it holds.
    pub fn key_count(&self) -> usize {
        self.starts.len()
    }

    /// Adds to `out` every ingredient with a text containing `word`.
    pub fn search_into(&self, word: &str, out: &mut Bits) {
        if word.is_empty() || word.contains('\0') {
            return;
        }
        let mut from = 0;
        while let Some(found) = self.text[from..].find(word) {
            let position = from + found;
            // The last start at or before the match is the text it is in.
            let key = self.starts.partition_point(|&s| s <= position) - 1;
            for &ingredient in &self.members[key] {
                out.set(ingredient as usize);
            }
            // The rest of this text can only repeat the same answer.
            match self.starts.get(key + 1) {
                Some(&next) => from = next,
                None => break,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn index(pairs: &[(&str, u32)]) -> TextIndex {
        let mut builder = TextIndexBuilder::default();
        for (text, ingredient) in pairs {
            builder.put(text, *ingredient);
        }
        builder.build()
    }

    fn search(index: &TextIndex, word: &str, len: usize) -> Vec<usize> {
        let mut out = Bits::empty(len);
        index.search_into(word, &mut out);
        (0..len).filter(|&n| out.contains(n)).collect()
    }

    // From JEI's LimitedStringStorageTest.backingIndexStoresEachKeyOnce.
    #[test]
    fn each_text_is_stored_once_and_finds_all_its_ingredients() {
        let index = index(&[("alpha", 1), ("alpha", 2)]);
        assert_eq!(index.key_count(), 1);
        assert_eq!(search(&index, "alpha", 4), [1, 2]);
    }

    // From JEI's ElementSearchTest: blank keys are skipped, others trimmed.
    #[test]
    fn blank_texts_are_skipped_and_others_trimmed() {
        let index = index(&[("", 0), (" \t ", 0), (" alpha ", 1)]);
        assert_eq!(index.key_count(), 1);
        assert_eq!(search(&index, "alpha", 2), [1]);
        assert_eq!(search(&index, " ", 2), Vec::<usize>::new());
    }

    #[test]
    fn a_word_matches_anywhere_inside_a_text_but_not_across_texts() {
        let index = index(&[("iron sword", 0), ("gold ingot", 1)]);
        assert_eq!(search(&index, "on sw", 2), [0]);
        assert_eq!(search(&index, "swordgold", 2), Vec::<usize>::new());
        assert_eq!(search(&index, "o", 2), [0, 1]);
    }

    #[test]
    fn the_empty_word_and_a_nul_find_nothing() {
        let index = index(&[("iron", 0)]);
        assert_eq!(search(&index, "", 1), Vec::<usize>::new());
        assert_eq!(search(&index, "\0", 1), Vec::<usize>::new());
    }

    #[test]
    fn multi_byte_text_is_searched_safely() {
        let index = index(&[("poção de cura", 0), ("日本語", 1)]);
        assert_eq!(search(&index, "ção", 2), [0]);
        assert_eq!(search(&index, "本", 2), [1]);
    }

    /// A small fixed generator: the same data in every run.
    struct Lcg(u64);
    impl Lcg {
        fn next(&mut self, below: usize) -> usize {
            self.0 = self
                .0
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            ((self.0 >> 33) as usize) % below
        }
    }

    #[test]
    fn it_answers_as_a_brute_force_search_does_on_random_data() {
        let mut random = Lcg(42);
        let alphabet = ['a', 'b', 'c', ' ', 'ç'];
        let word = |random: &mut Lcg, longest: usize| -> String {
            let length = 1 + random.next(longest);
            (0..length).map(|_| alphabet[random.next(5)]).collect()
        };
        let pairs: Vec<(String, u32)> = (0..500)
            .map(|n| (word(&mut random, 12), n as u32))
            .collect();
        let mut builder = TextIndexBuilder::default();
        for (text, n) in &pairs {
            builder.put(text, *n);
        }
        let index = builder.build();
        for _ in 0..500 {
            let query = word(&mut random, 4);
            let expected: Vec<usize> = pairs
                .iter()
                .filter(|(text, _)| text.trim().contains(query.as_str()))
                .map(|(_, n)| *n as usize)
                .collect();
            assert_eq!(search(&index, &query, 500), expected, "query {query:?}");
        }
    }

    #[test]
    fn bits_combine_as_sets() {
        let mut a = Bits::empty(130);
        a.set(1);
        a.set(129);
        a.set(500); // Out of range: ignored.
        let mut b = Bits::full(130);
        assert_eq!(b.count(), 130);
        b.clear(1);
        let mut both = a.clone();
        both.intersect_with(&b);
        assert_eq!((both.contains(1), both.contains(129)), (false, true));
        let mut either = a.clone();
        either.union_with(&b);
        assert_eq!(either.count(), 130);
        let mut rest = b.clone();
        rest.subtract(&a);
        assert_eq!(rest.count(), 128);
        assert!(Bits::empty(10).is_empty());
        assert!(!a.is_empty());
    }
}
