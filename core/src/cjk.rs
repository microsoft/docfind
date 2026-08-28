/// CJK (Chinese, Japanese, Korean) tokenization support.

pub fn is_cjk_char(c: char) -> bool {
    matches!(c,
        '\u{4E00}'..='\u{9FFF}'   // CJK Unified Ideographs
        | '\u{3400}'..='\u{4DBF}' // CJK Extension A
        | '\u{20000}'..='\u{2A6DF}' // CJK Extension B
        | '\u{3040}'..='\u{309F}' // Hiragana
        | '\u{30A0}'..='\u{30FF}' // Katakana
        | '\u{AC00}'..='\u{D7AF}' // Hangul Syllables
        | '\u{1100}'..='\u{11FF}' // Hangul Jamo
    )
}

/// Returns `true` if `c` is a Hangul character (Korean).
pub fn is_hangul_char(c: char) -> bool {
    matches!(c,
        '\u{AC00}'..='\u{D7AF}' // Hangul Syllables
        | '\u{1100}'..='\u{11FF}' // Hangul Jamo
    )
}

pub fn contains_cjk(text: &str) -> bool {
    text.chars().any(is_cjk_char)
}


/// Used to decide whether to apply the space-split strategy instead of n-grams.
pub fn is_predominantly_korean(text: &str) -> bool {
    let mut hangul = 0usize;
    let mut other_cjk = 0usize;
    for c in text.chars() {
        if is_hangul_char(c) {
            hangul += 1;
        } else if is_cjk_char(c) {
            other_cjk += 1;
        }
    }
    hangul > 0 && other_cjk == 0
}

// N-gram tokenizer for Chinese / Japanese

pub fn tokenize_cjk(text: &str) -> Vec<String> {
    let mut tokens: Vec<String> = Vec::new();

    let mut run: Vec<char> = Vec::new();

    let emit_run = |run: &[char], tokens: &mut Vec<String>| {
        match run.len() {
            0 => {}
            1 => {
                tokens.push(run[0].to_string());
            }
            2 => {
                let s: String = run.iter().collect();
                tokens.push(s);
            }
            n => {
                for i in 0..n - 1 {
                    let bigram: String = run[i..i + 2].iter().collect();
                    tokens.push(bigram);
                }
                if n >= 3 {
                    for i in 0..n - 2 {
                        let trigram: String = run[i..i + 3].iter().collect();
                        tokens.push(trigram);
                    }
                }
            }
        }
    };

    for c in text.chars() {
        if is_cjk_char(c) && !is_hangul_char(c) {
            run.push(c);
        } else if !run.is_empty() {
            emit_run(&run, &mut tokens);
            run.clear();
        }
    }
    if !run.is_empty() {
        emit_run(&run, &mut tokens);
    }

    
    tokens.into_iter().map(|t| t.to_lowercase()).collect()
}

// Korean tokenizer (space-split)

pub fn split_korean(text: &str) -> Vec<String> {
    text.split_whitespace()
        .filter(|tok| tok.chars().any(is_hangul_char))
        .map(|tok| {
            let trimmed = tok.trim_matches(|c: char| c.is_ascii_punctuation());
            trimmed.to_lowercase()
        })
        .filter(|tok| !tok.is_empty())
        .collect()
}

// Mixed-language dispatch

pub fn extract_cjk_keywords(text: &str) -> Vec<String> {
    if !contains_cjk(text) {
        return Vec::new();
    }

    let mut out: Vec<String> = Vec::new();

    // N-gram tokens for Chinese/Japanese characters
    let ngrams = tokenize_cjk(text);
    out.extend(ngrams);

    // Space-split tokens for Korean
    let korean = split_korean(text);
    out.extend(korean);

    // Deduplicate while preserving order
    let mut seen = std::collections::HashSet::new();
    out.retain(|t| seen.insert(t.clone()));

    out
}


