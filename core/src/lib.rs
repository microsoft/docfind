use serde::{Deserialize, Serialize};

#[cfg(any(feature = "cli", feature = "wasm", test))]
use std::collections::HashMap;

/// A minimal FSST-compressed vector of UTF-8 strings with random access.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FsstStrVec {
	// FSST dictionary we trained (as raw bytes for compact serde)
	dict_syms: Vec<[u8; 8]>,
	dict_lens: Vec<u8>,
	// Concatenated compressed payload and per-item offsets
	offsets: Vec<u32>, // offsets[i] = start of item i in `data`
	data: Vec<u8>,
}

impl FsstStrVec {
	/// Train FSST on `strings` and build the compressed vector.
	#[cfg(any(feature = "cli", test))]
	fn from_strings(strings: &[impl AsRef<str>]) -> Self {
		// 1) Train a compressor on the corpus.
		let sample: Vec<&[u8]> = strings.iter().map(|s| s.as_ref().as_bytes()).collect();
		let compressor = fsst::Compressor::train(&sample);

		// Keep dictionary for later decoding.
		let syms: Vec<fsst::Symbol> = compressor.symbol_table().to_vec();
		let lens: Vec<u8> = compressor.symbol_lengths().to_vec();

		// 2) Compress each string independently; store offsets + bytes.
		let mut offsets = Vec::with_capacity(strings.len());
		let mut data = Vec::new();
		for s in strings {
			offsets.push(data.len() as u32);
			let c = compressor.compress(s.as_ref().as_bytes());
			data.extend_from_slice(&c);
		}

		// 3) Store symbol table as raw bytes for compact serialization.
		let dict_syms: Vec<[u8; 8]> = syms
			.into_iter()
			.map(|sym| u64::to_le_bytes(sym.to_u64()))
			.collect();

		Self {
			dict_syms,
			dict_lens: lens,
			offsets,
			data,
		}
	}

	/// Number of strings
	pub fn len(&self) -> usize {
		self.offsets.len()
	}

	/// Random access: decode item i into an owned String.
	pub fn get(&self, i: usize) -> Option<String> {
		if i >= self.len() {
			return None;
		}
		let start = self.offsets[i] as usize;
		let end = if i + 1 < self.len() {
			self.offsets[i + 1] as usize
		} else {
			self.data.len()
		};
		let codes = &self.data[start..end];

		// Rebuild a Decompressor on-demand. (You can cache this in the struct if you
		// read frequently; it's cheap either way.)
		let syms: Vec<fsst::Symbol> = self
			.dict_syms
			.iter()
			.map(fsst::Symbol::from_slice)
			.collect();
		let decomp = fsst::Decompressor::new(&syms, &self.dict_lens);

		let bytes = decomp.decompress(codes);
		Some(String::from_utf8(bytes).expect("FSST preserves UTF-8 for UTF-8 input"))
	}
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
	pub title: String,
	pub category: String,
	pub href: String,
	pub body: String,
	pub keywords: Option<Vec<String>>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Index {
	/// FST vector for keyword to entry index
	fst: Vec<u8>,

	/// FSST string vector of all document strings
	document_strings: FsstStrVec,

	/// Vector of keyword to document index entries
	keyword_to_documents: Vec<Vec<(usize, u8)>>,
}

impl Index {
	pub fn from_bytes(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
		let index: Index = postcard::from_bytes(bytes)?;
		Ok(index)
	}

	pub fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
		Ok(postcard::to_allocvec(self)?)
	}
}

#[cfg(any(feature = "cli", test))]
pub fn build_index(documents: Vec<Document>) -> Result<Index, Box<dyn std::error::Error>> {
	use std::collections::HashSet;

	let stop_words = include_str!("../english.stop")
		.lines()
		.filter(|line| !line.is_empty() && !line.starts_with('#'))
		.map(|line| line.to_lowercase())
		.collect::<HashSet<String>>();

	let sw = rake::StopWords::from(stop_words);
	let rake = rake::Rake::new(sw.clone());

	let mut strings: Vec<&str> = Vec::new();
	let mut keywords_to_documents: HashMap<String, Vec<(&Document, f64)>> = HashMap::new();
	let mut doc_index_map: HashMap<&str, usize> = HashMap::new();

	for (doc_index, doc) in documents.iter().enumerate() {
		doc_index_map.insert(&doc.href, doc_index);
		strings.push(&doc.title);
		strings.push(&doc.category);
		strings.push(&doc.href);
		strings.push(&doc.body);

		let mut keyword_set: HashSet<String> = HashSet::new();
		let mut keywords: Vec<(String, f64)> = Vec::new();

		// Add explicit keywords from document metadata
		if let Some(kw) = &doc.keywords {
			for k in kw {
				let keyword = k
					.trim_matches(|c: char| !c.is_alphanumeric())
					.to_lowercase();
				if !keyword.is_empty() && !sw.contains(&keyword.clone()) && !keyword_set.contains(&keyword)
				{
					keywords.push((keyword.clone(), 100.0));
					keyword_set.insert(keyword.clone());
				}
			}
		}

		// add keywords from title
		let title_keywords = doc
			.title
			.split_whitespace()
			.map(|w| {
				w.trim_matches(|c: char| !c.is_alphanumeric())
					.to_lowercase()
			})
			.filter(|w| !w.is_empty() && !sw.contains(&w.clone()))
			.collect::<HashSet<String>>(); // deduplicate

		for tk in title_keywords {
			if !keyword_set.contains(&tk) {
				keywords.push((tk.clone(), 90.0));
				keyword_set.insert(tk.clone());
			}
		}

		let body_keywords = rake.run_fragments(vec![doc.body.as_str()]);
		let mut single_word_budget = 5;
		let mut double_word_budget = 3;

		for k in &body_keywords {
			let keyword = k.keyword.to_lowercase();

			// continue if keyword is already in title keywords
			if keyword_set.contains(&keyword) {
				continue;
			}

			let whitespace_count = k.keyword.matches(' ').count();

			if whitespace_count == 0 && single_word_budget > 0 {
				single_word_budget -= 1;
			} else if whitespace_count == 1 && double_word_budget > 0 {
				double_word_budget -= 1;
			} else {
				continue;
			}

			keywords.push((keyword.clone(), k.score));
			keyword_set.insert(keyword.clone());

			if single_word_budget == 0 && double_word_budget == 0 {
				break;
			}
		}

		for k in keywords.iter() {
			keywords_to_documents
				.entry(k.0.clone())
				.or_default()
				.push((doc, k.1));
		}
	}

	println!("Extracted {} unique keywords", keywords_to_documents.len());

	let mut fst_builder = fst::MapBuilder::memory();
	let mut keyword_to_documents: Vec<Vec<(usize, u8)>> = Vec::new();
	let mut keywords: Vec<String> = keywords_to_documents.keys().cloned().collect();
	keywords.sort();

	for (index, keyword) in keywords.iter().enumerate() {
		fst_builder.insert(keyword, index as u64)?;

		let mut doc_scores = keywords_to_documents.get(keyword).unwrap().clone();
		doc_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

		let entry = doc_scores
			.iter()
			.map(|(doc, score)| (doc_index_map[doc.href.as_str()], *score as u8))
			.collect::<Vec<(usize, u8)>>();

		keyword_to_documents.push(entry);
	}

	let fst = fst_builder.into_inner().unwrap();
	let document_strings = FsstStrVec::from_strings(&strings);

	Ok(Index {
		fst,
		document_strings,
		keyword_to_documents,
	})
}

#[cfg(any(feature = "wasm", test))]
pub fn search(
	index: &Index,
	query: &str,
	max_results: usize,
) -> Result<Vec<Document>, Box<dyn std::error::Error>> {
	use fst::automaton::Levenshtein;
	use fst::map::OpBuilder;
	use fst::{Automaton, Streamer};
	use std::collections::HashSet;

	let map = fst::Map::new(&index.fst)?;

	let mut query_words: HashSet<String> = query
		.split_whitespace()
		.map(|w| {
			w.trim_matches(|c: char| !c.is_alphanumeric())
				.to_lowercase()
		})
		.filter(|w| !w.is_empty())
		.collect();

	query_words.insert(query.to_lowercase());

	let mut keywords: Vec<(String, u64)> = Vec::new();

	for query_word in query_words {
		use fst::automaton::Str;

		let lev = Levenshtein::new(query_word.as_str(), 1)?;
		let prefix = Str::new(query_word.as_str()).starts_with();

		let mut op = OpBuilder::new()
			.add(map.search(lev))
			.add(map.search(prefix))
			.union();

		while let Some((keyword, indexed_value)) = op.next() {
			let keyword_str = String::from_utf8(keyword.to_vec())?;
			let score = indexed_value.to_vec().get(0).unwrap().value;
			keywords.push((keyword_str, score));
		}
	}

	// Sort keywords by length (shorter first)
	keywords.sort_by_key(|(kw, _)| kw.len());

	let mut documents: HashMap<usize, u8> = HashMap::new();

	for (_, keyword_index) in keywords {
		let documents_matching_keyword = &index.keyword_to_documents[keyword_index as usize];

		for (document_index, score) in documents_matching_keyword {
			let entry = documents.entry(*document_index).or_insert(0);
			*entry = entry.saturating_add(*score);
		}
	}

	// sort documents by score (descending), then by document index (ascending) for stable ordering
	let mut documents: Vec<(usize, u8)> = documents.into_iter().collect();
	documents.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
	documents.truncate(max_results);

	let mut result: Vec<Document> = Vec::new();

	for (document_index, _score) in documents {
		let title = index
			.document_strings
			.get(document_index * 4)
			.ok_or_else(|| "Failed to get document title")?;
		let category = index
			.document_strings
			.get(document_index * 4 + 1)
			.ok_or_else(|| "Failed to get document category")?;
		let href = index
			.document_strings
			.get(document_index * 4 + 2)
			.ok_or_else(|| "Failed to get document href")?;
		let body = index
			.document_strings
			.get(document_index * 4 + 3)
			.ok_or_else(|| "Failed to get document body")?;

		let document = Document {
			title,
			category,
			href,
			body,
			keywords: None,
		};

		result.push(document);
	}

	Ok(result)
}

#[cfg(test)]
mod tests;
