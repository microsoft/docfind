use docfind_core::Index;
use std::sync::OnceLock;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
	#[wasm_bindgen(js_namespace = console)]
	fn log(msg: &str);
}

#[unsafe(no_mangle)]
pub static mut INDEX_BASE: u32 = 0xdead_beef;

#[unsafe(no_mangle)]
pub static mut INDEX_LEN: u32 = 0xdead_beef;

static INDEX: OnceLock<Index> = OnceLock::new();

/// Search the index for a query string
/// Returns a JavaScript array of matching documents
#[wasm_bindgen]
pub fn search(query: &str, max_results: Option<usize>) -> Result<JsValue, JsValue> {
	let index = INDEX.get_or_init(|| {
		let raw_index =
			unsafe { std::slice::from_raw_parts(INDEX_BASE as *const u8, INDEX_LEN as usize) };
		Index::from_bytes(raw_index).expect("Failed to deserialize index")
	});

	let result = docfind_core::search(index, query, max_results.unwrap_or(10))
		.map_err(|e| JsValue::from_str(&format!("Search failed: {}", e)))?;

	serde_wasm_bindgen::to_value(&result)
		.map_err(|e| JsValue::from_str(&format!("Failed to convert results to JS: {}", e)))
}
