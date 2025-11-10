#[cfg(test)]
mod tests {
	use crate::Index;
	use crate::{Document, FsstStrVec};
	use crate::{build_index, search};

	// ========================================================================
	// SECTION 1: Basic Sanity Tests - FsstStrVec
	// ========================================================================

	#[test]
	#[cfg(feature = "cli")]
	fn test_fsst_str_vec_basic() {
		// Simple sanity test for FsstStrVec

		let strings = vec!["hello", "world", "rust", "search"];
		let vec = FsstStrVec::from_strings(&strings);

		assert_eq!(vec.len(), 4);

		assert_eq!(vec.get(0), Some("hello".to_string()));
		assert_eq!(vec.get(1), Some("world".to_string()));
		assert_eq!(vec.get(2), Some("rust".to_string()));
		assert_eq!(vec.get(3), Some("search".to_string()));
	}

	#[test]
	#[cfg(feature = "cli")]
	fn test_fsst_str_vec_out_of_bounds() {
		// Test that getting an out-of-bounds index returns None
		let strings = vec!["hello", "world"];
		let vec = FsstStrVec::from_strings(&strings);

		assert_eq!(vec.get(5), None);
		assert_eq!(vec.get(100), None);
	}

	#[test]
	#[cfg(feature = "cli")]
	fn test_fsst_str_vec_empty() {
		// Test behavior with empty vector
		let strings: Vec<&str> = vec![];
		let vec = FsstStrVec::from_strings(&strings);

		assert_eq!(vec.len(), 0);
		assert_eq!(vec.get(0), None);
	}

	#[test]
	#[cfg(feature = "cli")]
	fn test_fsst_str_vec_single_item() {
		// Test with a single string
		let strings = vec!["solo"];
		let vec = FsstStrVec::from_strings(&strings);

		assert_eq!(vec.len(), 1);
		assert_eq!(vec.get(0), Some("solo".to_string()));
		assert_eq!(vec.get(1), None);
	}

	#[test]
	#[cfg(feature = "cli")]
	fn test_fsst_str_vec_long_strings() {
		// Test with longer strings to verify compression works
		let strings = vec![
			"This is a much longer string that should compress well with FSST",
			"Another long string with similar patterns and repeated words",
			"The third long string continues the pattern with more text",
		];
		let vec = FsstStrVec::from_strings(&strings);

		assert_eq!(vec.len(), 3);
		assert_eq!(vec.get(0), Some(strings[0].to_string()));
		assert_eq!(vec.get(1), Some(strings[1].to_string()));
		assert_eq!(vec.get(2), Some(strings[2].to_string()));
	}

	#[test]
	#[cfg(feature = "cli")]
	fn test_fsst_str_vec_unicode() {
		// Test with Unicode strings
		let strings = vec!["Hello 世界", "Rust 🦀", "Café ☕"];
		let vec = FsstStrVec::from_strings(&strings);

		assert_eq!(vec.len(), 3);
		assert_eq!(vec.get(0), Some("Hello 世界".to_string()));
		assert_eq!(vec.get(1), Some("Rust 🦀".to_string()));
		assert_eq!(vec.get(2), Some("Café ☕".to_string()));
	}

	// ========================================================================
	// SECTION 2: Document Structure Tests
	// ========================================================================

	#[test]
	fn test_document_creation() {
		// Test that we can create Document structs
		let doc = Document {
			title: "Test Document".to_string(),
			category: "Test".to_string(),
			href: "/test".to_string(),
			body: "This is a test document body".to_string(),
			keywords: Some(vec!["test".to_string(), "document".to_string()]),
		};

		assert_eq!(doc.title, "Test Document");
		assert_eq!(doc.category, "Test");
		assert_eq!(doc.href, "/test");
		assert_eq!(doc.body, "This is a test document body");
		assert_eq!(
			doc.keywords,
			Some(vec!["test".to_string(), "document".to_string()])
		);
	}

	#[test]
	fn test_document_serialization() {
		// Test document serialization/deserialization
		let doc = Document {
			title: "Test".to_string(),
			category: "Category".to_string(),
			href: "/link".to_string(),
			body: "Body text".to_string(),
			keywords: Some(vec!["test".to_string(), "example".to_string()]),
		};

		let serialized = serde_json::to_string(&doc).unwrap();
		let deserialized: Document = serde_json::from_str(&serialized).unwrap();

		assert_eq!(doc.title, deserialized.title);
		assert_eq!(doc.category, deserialized.category);
		assert_eq!(doc.href, deserialized.href);
		assert_eq!(doc.body, deserialized.body);
		assert_eq!(doc.keywords, deserialized.keywords);
	}

	// ========================================================================
	// SECTION 3: Index Building Tests
	// ========================================================================

	#[test]
	#[cfg(feature = "cli")]
	fn test_build_index_simple() {
		// Test building a simple index with a few documents

		let documents = vec![
			Document {
				title: "Rust Programming".to_string(),
				category: "Documentation".to_string(),
				href: "/docs/rust".to_string(),
				body: "Learn Rust programming language".to_string(),
				keywords: Some(vec!["rust".to_string(), "programming".to_string()]),
			},
			Document {
				title: "Python Guide".to_string(),
				category: "Documentation".to_string(),
				href: "/docs/python".to_string(),
				body: "Python is a versatile programming language".to_string(),
				keywords: Some(vec!["python".to_string(), "guide".to_string()]),
			},
		];

		let index = build_index(documents);
		assert!(index.is_ok());

		let index = index.unwrap();
		assert_eq!(index.document_strings.len(), 8); // 4 strings per document * 2 documents
	}

	#[test]
	#[cfg(feature = "cli")]
	fn test_build_index_empty() {
		// Test building an index with no documents
		let documents: Vec<Document> = vec![];
		let index = build_index(documents);
		assert!(index.is_ok());

		let index = index.unwrap();
		assert_eq!(index.document_strings.len(), 0);
	}

	#[test]
	#[cfg(feature = "cli")]
	fn test_build_index_single_document() {
		// Test building an index with a single document
		let documents = vec![Document {
			title: "Single Document".to_string(),
			category: "Test".to_string(),
			href: "/single".to_string(),
			body: "This is the only document".to_string(),
			keywords: Some(vec!["single".to_string(), "document".to_string()]),
		}];

		let index = build_index(documents);
		assert!(index.is_ok());

		let index = index.unwrap();
		assert_eq!(index.document_strings.len(), 4); // title, category, href, body
	}

	#[test]
	#[cfg(feature = "cli")]
	fn test_build_index_duplicate_titles() {
		// Test with documents that have similar or duplicate titles
		let documents = vec![
			Document {
				title: "Getting Started".to_string(),
				category: "Guide".to_string(),
				href: "/guide1".to_string(),
				body: "First guide".to_string(),
				keywords: Some(vec!["getting".to_string(), "started".to_string()]),
			},
			Document {
				title: "Getting Started".to_string(),
				category: "Tutorial".to_string(),
				href: "/tutorial1".to_string(),
				body: "First tutorial".to_string(),
				keywords: Some(vec!["getting".to_string(), "started".to_string()]),
			},
		];

		let index = build_index(documents);
		assert!(index.is_ok());
	}

	// ========================================================================
	// SECTION 4: Index Serialization Tests
	// ========================================================================

	#[test]
	#[cfg(feature = "cli")]
	fn test_index_serialization() {
		// Test that we can serialize and deserialize an index

		let documents = vec![Document {
			title: "Test Document".to_string(),
			category: "Test".to_string(),
			href: "/test".to_string(),
			body: "This is a test document".to_string(),
			keywords: Some(vec!["test".to_string(), "document".to_string()]),
		}];

		let index = build_index(documents).unwrap();

		// Create a buffer to serialize to
		let buffer = index.to_bytes().unwrap();
		assert!(!buffer.is_empty());

		// Try to deserialize from the buffer
		let deserialized = Index::from_bytes(&buffer);
		assert!(deserialized.is_ok());

		let deserialized_index = deserialized.unwrap();
		assert_eq!(
			deserialized_index.document_strings.len(),
			index.document_strings.len()
		);
	}

	#[test]
	#[cfg(feature = "cli")]
	fn test_index_serialization_roundtrip() {
		// Test that we can serialize and deserialize multiple times
		let documents = vec![
			Document {
				title: "Document One".to_string(),
				category: "Category A".to_string(),
				href: "/doc1".to_string(),
				body: "Content for document one".to_string(),
				keywords: Some(vec!["document".to_string(), "one".to_string()]),
			},
			Document {
				title: "Document Two".to_string(),
				category: "Category B".to_string(),
				href: "/doc2".to_string(),
				body: "Content for document two".to_string(),
				keywords: Some(vec!["document".to_string(), "two".to_string()]),
			},
		];

		let original_index = build_index(documents).unwrap();

		// First roundtrip
		let buffer1 = original_index.to_bytes().unwrap();
		let index1 = Index::from_bytes(&buffer1).unwrap();

		// Second roundtrip
		let buffer2 = index1.to_bytes().unwrap();
		let index2 = Index::from_bytes(&buffer2).unwrap();

		// Verify the data is consistent
		assert_eq!(
			index2.document_strings.len(),
			original_index.document_strings.len()
		);
	}

	// ========================================================================
	// SECTION 5: Simple Search Tests
	// ========================================================================

	#[test]
	#[cfg(all(feature = "cli", feature = "wasm"))]
	fn test_search_single_word() {
		// Test searching for a single word
		let documents = vec![
			Document {
				title: "Rust Programming".to_string(),
				category: "Documentation".to_string(),
				href: "/docs/rust".to_string(),
				body: "Learn Rust programming language".to_string(),
				keywords: Some(vec!["rust".to_string(), "programming".to_string()]),
			},
			Document {
				title: "Python Guide".to_string(),
				category: "Documentation".to_string(),
				href: "/docs/python".to_string(),
				body: "Python is a versatile programming language".to_string(),
				keywords: Some(vec!["python".to_string(), "guide".to_string()]),
			},
		];

		let index = build_index(documents).unwrap();
		let results = search(&index, "Rust", 10).unwrap();

		assert!(!results.is_empty());
		assert_eq!(results[0].title, "Rust Programming");
		assert_eq!(results[0].href, "/docs/rust");
	}

	#[test]
	#[cfg(all(feature = "cli", feature = "wasm"))]
	fn test_search_case_insensitive() {
		// Test that search is case-insensitive
		let documents = vec![Document {
			title: "JavaScript Tutorial".to_string(),
			category: "Tutorials".to_string(),
			href: "/tutorials/javascript".to_string(),
			body: "Learn JavaScript programming".to_string(),
			keywords: Some(vec!["javascript".to_string(), "tutorial".to_string()]),
		}];

		let index = build_index(documents).unwrap();

		let results_lower = search(&index, "javascript", 10).unwrap();
		let results_upper = search(&index, "JAVASCRIPT", 10).unwrap();
		let results_mixed = search(&index, "JavaScript", 10).unwrap();

		assert!(!results_lower.is_empty());
		assert!(!results_upper.is_empty());
		assert!(!results_mixed.is_empty());

		// All should find the same document
		assert_eq!(results_lower[0].href, "/tutorials/javascript");
		assert_eq!(results_upper[0].href, "/tutorials/javascript");
		assert_eq!(results_mixed[0].href, "/tutorials/javascript");
	}

	#[test]
	#[cfg(all(feature = "cli", feature = "wasm"))]
	fn test_search_no_results() {
		// Test searching for something that doesn't exist
		let documents = vec![Document {
			title: "Rust Programming".to_string(),
			category: "Documentation".to_string(),
			href: "/docs/rust".to_string(),
			body: "Learn Rust programming language".to_string(),
			keywords: Some(vec!["rust".to_string(), "programming".to_string()]),
		}];

		let index = build_index(documents).unwrap();
		let results = search(&index, "NonexistentKeyword", 10).unwrap();

		assert!(results.is_empty());
	}

	#[test]
	#[cfg(all(feature = "cli", feature = "wasm"))]
	fn test_search_empty_query() {
		// Test searching with an empty query
		let documents = vec![Document {
			title: "Test Document".to_string(),
			category: "Test".to_string(),
			href: "/test".to_string(),
			body: "Test content".to_string(),
			keywords: Some(vec!["test".to_string(), "document".to_string()]),
		}];

		let index = build_index(documents).unwrap();
		let results = search(&index, "", 10).unwrap();

		// Empty query should return no results (or possibly all results depending on implementation)
		// Just verify it doesn't crash
		assert!(results.len() <= 1);
	}

	// ========================================================================
	// SECTION 6: Multi-word and Phrase Search Tests
	// ========================================================================

	#[test]
	#[cfg(all(feature = "cli", feature = "wasm"))]
	fn test_search_multiple_words() {
		// Test searching for multiple words
		let documents = vec![
			Document {
				title: "VS Code Extensions".to_string(),
				category: "Documentation".to_string(),
				href: "/docs/extensions".to_string(),
				body: "Learn how to create VS Code extensions with comprehensive guides".to_string(),
				keywords: Some(vec![
					"vs".to_string(),
					"code".to_string(),
					"extensions".to_string(),
				]),
			},
			Document {
				title: "VS Code Settings".to_string(),
				category: "Documentation".to_string(),
				href: "/docs/settings".to_string(),
				body: "Configure your VS Code settings for optimal development experience".to_string(),
				keywords: Some(vec![
					"vs".to_string(),
					"code".to_string(),
					"settings".to_string(),
				]),
			},
			Document {
				title: "Python Guide".to_string(),
				category: "Documentation".to_string(),
				href: "/docs/python".to_string(),
				body: "Python is a versatile programming language".to_string(),
				keywords: Some(vec!["python".to_string(), "guide".to_string()]),
			},
		];

		let index = build_index(documents).unwrap();
		let results = search(&index, "VS Code", 10).unwrap();

		// Should find both VS Code documents
		assert!(results.len() >= 2);
		assert!(results.iter().any(|d| d.href == "/docs/extensions"));
		assert!(results.iter().any(|d| d.href == "/docs/settings"));
	}

	#[test]
	#[cfg(all(feature = "cli", feature = "wasm"))]
	fn test_search_partial_word_match() {
		// Test that partial word matches work
		let documents = vec![Document {
			title: "Debugging in VS Code".to_string(),
			category: "Documentation".to_string(),
			href: "/docs/debugging".to_string(),
			body: "Debug your applications with powerful debugging tools".to_string(),
			keywords: Some(vec![
				"debugging".to_string(),
				"vs".to_string(),
				"code".to_string(),
			]),
		}];

		let index = build_index(documents).unwrap();
		let results = search(&index, "debug", 10).unwrap();

		// Should find documents with "debugging" and "debug"
		assert!(!results.is_empty());
	}

	// ========================================================================
	// SECTION 7: Ranking and Relevance Tests
	// ========================================================================

	#[test]
	#[cfg(all(feature = "cli", feature = "wasm"))]
	fn test_search_title_match_ranks_higher() {
		// Test that title matches rank higher than body matches
		let documents = vec![
			Document {
				title: "Python Tutorial".to_string(),
				category: "Tutorials".to_string(),
				href: "/tutorials/python".to_string(),
				body: "Learn programming with this tutorial".to_string(),
				keywords: Some(vec!["python".to_string(), "tutorial".to_string()]),
			},
			Document {
				title: "Getting Started".to_string(),
				category: "Documentation".to_string(),
				href: "/docs/start".to_string(),
				body: "This guide covers Python basics and advanced features".to_string(),
				keywords: Some(vec!["getting".to_string(), "started".to_string()]),
			},
		];

		let index = build_index(documents).unwrap();
		let results = search(&index, "Python", 10).unwrap();

		// Document with "Python" in title should rank first
		assert!(!results.is_empty());
		assert_eq!(results[0].href, "/tutorials/python");
	}

	#[test]
	#[cfg(all(feature = "cli", feature = "wasm"))]
	fn test_search_multiple_keyword_matches() {
		// Test that documents matching multiple keywords rank higher
		let documents = vec![
			Document {
				title: "VS Code Debugging".to_string(),
				category: "Documentation".to_string(),
				href: "/docs/debugging".to_string(),
				body: "Debug VS Code extensions".to_string(),
				keywords: Some(vec![
					"vs".to_string(),
					"code".to_string(),
					"debugging".to_string(),
				]),
			},
			Document {
				title: "VS Code Overview".to_string(),
				category: "Documentation".to_string(),
				href: "/docs/overview".to_string(),
				body: "Introduction to the editor".to_string(),
				keywords: Some(vec![
					"vs".to_string(),
					"code".to_string(),
					"overview".to_string(),
				]),
			},
			Document {
				title: "Debugging Guide".to_string(),
				category: "Tutorials".to_string(),
				href: "/tutorials/debug".to_string(),
				body: "General debugging techniques".to_string(),
				keywords: Some(vec!["debugging".to_string(), "guide".to_string()]),
			},
		];

		let index = build_index(documents).unwrap();
		let results = search(&index, "VS Code debugging", 10).unwrap();

		// Document with all three keywords should rank first
		assert!(!results.is_empty());
		assert_eq!(results[0].href, "/docs/debugging");
	}

	#[test]
	#[cfg(all(feature = "cli", feature = "wasm"))]
	fn test_search_max_results_limit() {
		// Test that max_results parameter limits results correctly
		let documents = vec![
			Document {
				title: "Guide One".to_string(),
				category: "Guides".to_string(),
				href: "/guide1".to_string(),
				body: "First guide about programming".to_string(),
				keywords: Some(vec!["guide".to_string(), "one".to_string()]),
			},
			Document {
				title: "Guide Two".to_string(),
				category: "Guides".to_string(),
				href: "/guide2".to_string(),
				body: "Second guide about programming".to_string(),
				keywords: Some(vec!["guide".to_string(), "two".to_string()]),
			},
			Document {
				title: "Guide Three".to_string(),
				category: "Guides".to_string(),
				href: "/guide3".to_string(),
				body: "Third guide about programming".to_string(),
				keywords: Some(vec!["guide".to_string(), "three".to_string()]),
			},
			Document {
				title: "Guide Four".to_string(),
				category: "Guides".to_string(),
				href: "/guide4".to_string(),
				body: "Fourth guide about programming".to_string(),
				keywords: Some(vec!["guide".to_string(), "four".to_string()]),
			},
		];

		let index = build_index(documents).unwrap();

		let results_2 = search(&index, "guide", 2).unwrap();
		let results_3 = search(&index, "guide", 3).unwrap();
		let results_10 = search(&index, "guide", 10).unwrap();

		assert!(results_2.len() <= 2);
		assert!(results_3.len() <= 3);
		assert!(results_10.len() <= 10);
	}

	// ========================================================================
	// SECTION 8: Complex Search Query Tests
	// ========================================================================

	#[test]
	#[cfg(all(feature = "cli", feature = "wasm"))]
	fn test_search_technical_terms() {
		// Test searching for technical terms and acronyms
		let documents = vec![
			Document {
				title: "TypeScript Configuration".to_string(),
				category: "Documentation".to_string(),
				href: "/docs/typescript".to_string(),
				body: "Configure TypeScript with tsconfig.json for your project".to_string(),
				keywords: Some(vec!["typescript".to_string(), "configuration".to_string()]),
			},
			Document {
				title: "JavaScript Basics".to_string(),
				category: "Tutorials".to_string(),
				href: "/tutorials/javascript".to_string(),
				body: "Learn JavaScript fundamentals".to_string(),
				keywords: Some(vec!["javascript".to_string(), "basics".to_string()]),
			},
			Document {
				title: "Language Support".to_string(),
				category: "Documentation".to_string(),
				href: "/docs/languages".to_string(),
				body: "VS Code supports TypeScript, JavaScript, and many other languages".to_string(),
				keywords: Some(vec!["language".to_string(), "support".to_string()]),
			},
		];

		let index = build_index(documents).unwrap();
		let results = search(&index, "TypeScript", 10).unwrap();

		assert!(!results.is_empty());
		assert!(results.iter().any(|d| d.href == "/docs/typescript"));
	}

	#[test]
	#[cfg(all(feature = "cli", feature = "wasm"))]
	fn test_search_with_special_characters() {
		// Test searching with special characters
		let documents = vec![
			Document {
				title: "C++ Programming".to_string(),
				category: "Documentation".to_string(),
				href: "/docs/cpp".to_string(),
				body: "Learn C++ programming language".to_string(),
				keywords: Some(vec!["c++".to_string(), "programming".to_string()]),
			},
			Document {
				title: "C# Guide".to_string(),
				category: "Documentation".to_string(),
				href: "/docs/csharp".to_string(),
				body: "C# development with .NET".to_string(),
				keywords: Some(vec!["c#".to_string(), "guide".to_string()]),
			},
		];

		let index = build_index(documents).unwrap();
		let results_cpp = search(&index, "C++", 10);
		let results_csharp = search(&index, "C#", 10);

		// Should handle special characters gracefully
		assert!(results_cpp.is_ok() as bool);
		assert!(results_csharp.is_ok() as bool);
	}

	#[test]
	#[cfg(all(feature = "cli", feature = "wasm"))]
	fn test_search_compound_keywords() {
		// Test searching for compound keywords and multi-word phrases
		let documents = vec![
			Document {
				title: "Remote Development Setup".to_string(),
				category: "Tutorials".to_string(),
				href: "/tutorials/remote-dev".to_string(),
				body: "Set up remote development environment for distributed teams".to_string(),
				keywords: Some(vec![
					"remote".to_string(),
					"development".to_string(),
					"setup".to_string(),
				]),
			},
			Document {
				title: "Development Environment".to_string(),
				category: "Documentation".to_string(),
				href: "/docs/environment".to_string(),
				body: "Configure your local development environment".to_string(),
				keywords: Some(vec!["development".to_string(), "environment".to_string()]),
			},
			Document {
				title: "Remote Connections".to_string(),
				category: "Documentation".to_string(),
				href: "/docs/remote".to_string(),
				body: "Connect to remote servers and containers".to_string(),
				keywords: Some(vec!["remote".to_string(), "connections".to_string()]),
			},
		];

		let index = build_index(documents).unwrap();
		let results = search(&index, "remote development", 10).unwrap();

		// Should find the document that has both keywords together
		assert!(!results.is_empty());
		assert_eq!(results[0].href, "/tutorials/remote-dev");
	}

	#[test]
	#[cfg(all(feature = "cli", feature = "wasm"))]
	fn test_search_with_stopwords() {
		// Test that common stop words don't interfere with search
		let documents = vec![Document {
			title: "Getting Started with VS Code".to_string(),
			category: "Tutorials".to_string(),
			href: "/tutorials/start".to_string(),
			body: "This is a guide to help you get started with the editor".to_string(),
			keywords: Some(vec![
				"getting".to_string(),
				"started".to_string(),
				"vs".to_string(),
				"code".to_string(),
			]),
		}];

		let index = build_index(documents).unwrap();
		let results = search(&index, "getting started with vscode", 10).unwrap();

		// Should find results despite stop words like "with", "the", "a"
		assert!(!results.is_empty());
	}

	#[test]
	#[cfg(all(feature = "cli", feature = "wasm"))]
	fn test_search_with_numbers() {
		// Test searching with version numbers and numeric values
		let documents = vec![
			Document {
				title: "Node.js 18 Features".to_string(),
				category: "Updates".to_string(),
				href: "/updates/nodejs18".to_string(),
				body: "New features in Node.js version 18 release".to_string(),
				keywords: Some(vec![
					"node.js".to_string(),
					"18".to_string(),
					"features".to_string(),
				]),
			},
			Document {
				title: "Node.js 16 Support".to_string(),
				category: "Updates".to_string(),
				href: "/updates/nodejs16".to_string(),
				body: "Long-term support for Node.js 16".to_string(),
				keywords: Some(vec![
					"node.js".to_string(),
					"16".to_string(),
					"support".to_string(),
				]),
			},
		];

		let index = build_index(documents).unwrap();
		let results = search(&index, "nodejs 18", 10).unwrap();

		assert!(!results.is_empty());
		// Should find the Node.js 18 document
		assert!(results.iter().any(|d| d.href.contains("nodejs18")));
	}

	#[test]
	#[cfg(all(feature = "cli", feature = "wasm"))]
	fn test_search_long_query() {
		// Test with a longer, more natural language query
		let documents = vec![
			Document {
				title: "Remote SSH Extension".to_string(),
				category: "Extensions".to_string(),
				href: "/extensions/remote-ssh".to_string(),
				body: "Connect to remote servers via SSH and develop directly on remote machines"
					.to_string(),
				keywords: Some(vec![
					"remote".to_string(),
					"ssh".to_string(),
					"extension".to_string(),
				]),
			},
			Document {
				title: "SSH Key Setup".to_string(),
				category: "Documentation".to_string(),
				href: "/docs/ssh-keys".to_string(),
				body: "Configure SSH keys for secure remote connections".to_string(),
				keywords: Some(vec![
					"ssh".to_string(),
					"key".to_string(),
					"setup".to_string(),
				]),
			},
		];

		let index = build_index(documents).unwrap();
		let results = search(&index, "how do i connect to a remote server using ssh", 10).unwrap();

		// Should extract relevant keywords and find documents
		assert!(!results.is_empty());
	}

	// ========================================================================
	// SECTION 9: Edge Cases and Stress Tests
	// ========================================================================

	#[test]
	#[cfg(all(feature = "cli", feature = "wasm"))]
	fn test_search_many_documents() {
		// Test with a larger number of documents
		let mut documents = Vec::new();
		for i in 0..100 {
			documents.push(Document {
				title: format!("Document {}", i).to_string(),
				category: format!("Category {}", i % 10).to_string(),
				href: format!("/doc{}", i).to_string(),
				body: format!("This is document number {} with some content", i).to_string(),
				keywords: Some(vec![format!("document{}", i).to_string()]),
			});
		}

		// Add a special document to search for
		documents.push(Document {
			title: "Special Search Target".to_string(),
			category: "Test".to_string(),
			href: "/special".to_string(),
			body: "This document should be easy to find".to_string(),
			keywords: Some(vec!["special".to_string(), "target".to_string()]),
		});

		let index = build_index(documents).unwrap();
		let results = search(&index, "special target", 10).unwrap();

		assert!(!results.is_empty());
		assert_eq!(results[0].href, "/special");
	}

	#[test]
	#[cfg(all(feature = "cli", feature = "wasm"))]
	fn test_search_empty_fields() {
		// Test with documents that have empty fields
		let documents = vec![
			Document {
				title: "".to_string(),
				category: "Empty Title".to_string(),
				href: "/empty1".to_string(),
				body: "This document has no title".to_string(),
				keywords: Some(vec!["empty".to_string()]),
			},
			Document {
				title: "Empty Body".to_string(),
				category: "Test".to_string(),
				href: "/empty2".to_string(),
				body: "".to_string(),
				keywords: Some(vec!["empty".to_string(), "body".to_string()]),
			},
		];

		let index = build_index(documents);
		assert!(index.is_ok());

		let results = search(&index.unwrap(), "empty", 10).unwrap();
		// Should handle empty fields gracefully
		assert!(!results.is_empty());
	}

	#[test]
	#[cfg(all(feature = "cli", feature = "wasm"))]
	fn test_search_whitespace_handling() {
		// Test that extra whitespace doesn't break search
		let documents = vec![Document {
			title: "Whitespace   Test".to_string(),
			category: "Test".to_string(),
			href: "/whitespace".to_string(),
			body: "Multiple   spaces   between   words".to_string(),
			keywords: Some(vec!["whitespace".to_string(), "test".to_string()]),
		}];

		let index = build_index(documents).unwrap();
		let results = search(&index, "  whitespace  test  ", 10).unwrap();

		assert!(!results.is_empty());
	}

	#[test]
	#[cfg(all(feature = "cli", feature = "wasm"))]
	fn test_search_with_typo() -> Result<(), Box<dyn std::error::Error>> {
		let document_strings = FsstStrVec::from_strings(&vec![
			"Document 1",
			"Docs",
			"/doc1",
			"This is the first document.",
			"Document 2",
			"Docs",
			"/doc2",
			"This is the second document.",
			"Document 3",
			"Docs",
			"/doc3",
			"This is the third document.",
		]);

		let keyword_to_documents: Vec<Vec<(usize, u8)>> = vec![
			vec![(1, 1)],          // "language" appears in doc 1
			vec![(0, 10), (2, 4)], // "programming" appears in doc 0 and 2
			vec![(0, 5), (1, 3)],  // "rust" appears in doc 0 and 1
		];

		let mut fst_builder = fst::MapBuilder::memory();
		fst_builder.insert("language", 0).unwrap();
		fst_builder.insert("programming", 1).unwrap();
		fst_builder.insert("rust", 2).unwrap();
		let fst = fst_builder.into_inner()?;

		let index = Index {
			fst,
			document_strings,
			keyword_to_documents,
		};

		let results = search(&index, "lamguage", 10)?;
		assert_eq!(results.len(), 1, "Expected 1 result for 'lamguage'");

		Ok(())
	}
}
