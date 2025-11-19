use docfind_core::Document;
use std::io::Write;
use std::path::Path;
use std::{collections::HashMap, fs::File};
use wasm_encoder::{ConstExpr, DataSection, MemorySection, MemoryType};
use wasmparser::{Parser, Payload};

#[derive(Debug)]
enum WasmDataSegment {
	Passive(Vec<u8>),
	Active {
		memory_index: u32,
		offset: ConstExpr,
		data: Vec<u8>,
		i32const_offset: Option<i32>,
	},
}

/// Represents different types of WASM sections we care about
#[derive(Debug)]
enum WasmSection {
	Data(Vec<WasmDataSegment>),
	DataCount(u32),
	Memory,
	Raw { id: u8, data: Vec<u8> },
}

/// Convert a wasmparser ConstExpr to a wasm_encoder ConstExpr
fn convert_const_expr(
	expr: &wasmparser::ConstExpr,
) -> Result<ConstExpr, Box<dyn std::error::Error>> {
	let mut ops_reader = expr.get_operators_reader();

	// We'll handle the most common cases
	if !ops_reader.eof() {
		let op = ops_reader.read()?;
		match op {
			wasmparser::Operator::I32Const { value } => return Ok(ConstExpr::i32_const(value)),
			wasmparser::Operator::I64Const { value } => return Ok(ConstExpr::i64_const(value)),
			wasmparser::Operator::F32Const { value } => {
				// Convert wasmparser Ieee32 to wasm_encoder Ieee32
				let f32_val = f32::from_bits(value.bits());
				return Ok(ConstExpr::f32_const(f32_val.into()));
			}
			wasmparser::Operator::F64Const { value } => {
				// Convert wasmparser Ieee64 to wasm_encoder Ieee64
				let f64_val = f64::from_bits(value.bits());
				return Ok(ConstExpr::f64_const(f64_val.into()));
			}
			wasmparser::Operator::GlobalGet { global_index } => {
				return Ok(ConstExpr::global_get(global_index));
			}
			wasmparser::Operator::RefNull { hty } => {
				// Convert heap type
				let heap_type = match hty {
					wasmparser::HeapType::Concrete(_) => wasm_encoder::HeapType::Concrete(0),
					_ => wasm_encoder::HeapType::Abstract {
						shared: false,
						ty: wasm_encoder::AbstractHeapType::Func,
					},
				};
				return Ok(ConstExpr::ref_null(heap_type));
			}
			wasmparser::Operator::RefFunc { function_index } => {
				return Ok(ConstExpr::ref_func(function_index));
			}
			_ => {
				// For other operators, use raw with empty bytes
				return Ok(ConstExpr::raw(vec![]));
			}
		}
	}

	Ok(ConstExpr::raw(vec![]))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let debug = std::env::var("DOCFIND_DEBUG").is_ok();
	let args: Vec<String> = std::env::args().collect();

	if args.len() != 3 {
		eprintln!("Usage: {} <documents.json> <outdir>", args[0]);
		std::process::exit(1);
	}

	let input_path = &args[1];
	let output_dir = &args[2];
	if debug {
		eprintln!("[docfind] CWD: {:?}", std::env::current_dir()?);
		eprintln!("[docfind] input_path: {}", input_path);
		eprintln!("[docfind] output_dir: {}", output_dir);
	}
	let documents_file = File::open(input_path)?;
	let documents: Vec<Document> = serde_json::from_reader(documents_file)?;

	let start = std::time::Instant::now();
	let index = docfind_core::build_index(documents)?;
	let duration = start.elapsed();
	if debug {
		eprintln!("[docfind] Indexing completed in: {:?}", duration);
	} else {
		println!("Indexing completed in: {:?}", duration);
	}

	let start = std::time::Instant::now();
	let mut sections: Vec<WasmSection> = Vec::new();

	let mut old_memory_page_count: u64 = 0;
	let mut index_base_global_index: Option<u32> = None;
	let mut index_len_global_index: Option<u32> = None;
	let mut i32_globals: HashMap<u32, i32> = HashMap::new();

	let docfind_js: &[u8] = include_bytes!("../../wasm/pkg/docfind.js");
	let docfind_bg_wasm: &[u8] = include_bytes!("../../wasm/pkg/docfind_bg.wasm");
	if debug {
		eprintln!("[docfind] Embedded JS size: {} bytes", docfind_js.len());
		eprintln!(
			"[docfind] Embedded WASM size: {} bytes",
			docfind_bg_wasm.len()
		);
	}

	for payload in Parser::new(0).parse_all(docfind_bg_wasm) {
		let payload = payload?;

		// process i32 const data sections differently
		if let Payload::DataSection(reader) = payload {
			let mut data_segments: Vec<WasmDataSegment> = Vec::new();

			for data in reader {
				let data = data?;

				match data.kind {
					wasmparser::DataKind::Passive => {
						data_segments.push(WasmDataSegment::Passive(data.data.to_vec()));
					}
					wasmparser::DataKind::Active {
						memory_index,
						offset_expr,
					} => {
						let const_expr = convert_const_expr(&offset_expr)?;
						let i32const_offset = if let wasmparser::Operator::I32Const { value } =
							offset_expr.get_operators_reader().read()?
						{
							Some(value)
						} else {
							None
						};

						data_segments.push(WasmDataSegment::Active {
							memory_index,
							offset: const_expr,
							data: data.data.to_vec(),
							i32const_offset,
						});
					}
				}
			}

			sections.push(WasmSection::Data(data_segments));
		} else if let Payload::DataCountSection { count, .. } = payload {
			sections.push(WasmSection::DataCount(count));
		} else if let Payload::MemorySection(reader) = payload {
			for memory in reader {
				old_memory_page_count = memory?.initial as u64;
			}
			sections.push(WasmSection::Memory);
		} else {
			if let Some((id, data)) = payload.as_section() {
				sections.push(WasmSection::Raw {
					id,
					data: docfind_bg_wasm[data.start..data.end].to_vec(),
				});
			}

			match payload {
				Payload::ExportSection(reader) => {
					for export in reader {
						let export = export?;
						if export.name == "INDEX_BASE" {
							index_base_global_index = Some(export.index);
						} else if export.name == "INDEX_LEN" {
							index_len_global_index = Some(export.index);
						}
					}
				}
				Payload::GlobalSection(reader) => {
					for (idx, global) in reader.into_iter().enumerate() {
						let global = global?;
						let mut ops_reader = global.init_expr.get_operators_reader();

						if !ops_reader.eof() {
							if let Ok(wasmparser::Operator::I32Const { value }) = ops_reader.read() {
								i32_globals.insert(idx as u32, value);
							}
						}
					}
				}
				_ => {}
			}
		}
	}

	let index_base_global_index =
		index_base_global_index.expect("Could not find INDEX_BASE global index");
	let index_len_global_index =
		index_len_global_index.expect("Could not find INDEX_LEN global index");
	if debug {
		eprintln!(
			"[docfind] INDEX_BASE global index: {}",
			index_base_global_index
		);
		eprintln!(
			"[docfind] INDEX_LEN global index: {}",
			index_len_global_index
		);
	}

	let index_base_global_address = i32_globals
		.get(&index_base_global_index)
		.expect("Could not find INDEX_BASE global value");

	let index_len_global_address = i32_globals
		.get(&index_len_global_index)
		.expect("Could not find INDEX_LEN global value");
	if debug {
		eprintln!(
			"[docfind] INDEX_BASE address: {}",
			index_base_global_address
		);
		eprintln!("[docfind] INDEX_LEN address: {}", index_len_global_address);
	}

	let raw_index: Vec<u8> = index.to_bytes()?; // will embed into wasm
	if debug {
		eprintln!("[docfind] Index size: {} bytes", raw_index.len());
	} else {
		println!("Index size: {} bytes", raw_index.len());
	}

	let new_memory_page_count = old_memory_page_count + (raw_index.len() as u64 / 0x10000) + 1;
	let index_base = old_memory_page_count * 0x10000;
	if debug {
		eprintln!("[docfind] Old memory pages: {}", old_memory_page_count);
		eprintln!("[docfind] New memory pages: {}", new_memory_page_count);
		eprintln!("[docfind] Index base address: {}", index_base);
	}

	let mut encoder = wasm_encoder::Module::new();

	for section in sections {
		match section {
			WasmSection::DataCount(count) => {
				encoder.section(&wasm_encoder::DataCountSection { count: count + 1 });
			}
			WasmSection::Data(data_segments) => {
				let mut data_section = DataSection::new();

				for segment in data_segments {
					match segment {
						WasmDataSegment::Passive(data) => {
							data_section.passive(data.iter().copied());
						}
						WasmDataSegment::Active {
							memory_index,
							offset,
							data,
							i32const_offset,
						} => {
							if let Some(i32_offset) = i32const_offset {
								let start = i32_offset;
								let end = i32_offset + (data.len() as i32);

								// Patch the data if it contains the INDEX_BASE or INDEX_LEN addresses
								if index_base_global_address >= &start && index_base_global_address < &end {
									assert!(
										index_len_global_address >= &start && index_len_global_address < &end,
										"INDEX_LEN address not in data segment!"
									);

									let mut data = data;

									let base_relative_offset = (index_base_global_address - start) as usize;
									data[base_relative_offset..base_relative_offset + 4]
										.copy_from_slice(&(index_base as i32).to_le_bytes());

									let length_relative_offset = (index_len_global_address - start) as usize;
									data[length_relative_offset..length_relative_offset + 4]
										.copy_from_slice(&(raw_index.len() as i32).to_le_bytes());

									data_section.active(memory_index, &offset, data);
									continue;
								}
							}

							data_section.active(memory_index, &offset, data);
						}
					}
				}

				data_section.active(
					0,
					&ConstExpr::i32_const(index_base as i32),
					raw_index.iter().copied(),
				);

				encoder.section(&data_section);
			}
			WasmSection::Memory => {
				let mut new_memory_section = MemorySection::new();
				new_memory_section.memory(MemoryType {
					minimum: new_memory_page_count,
					maximum: None,
					memory64: false,
					shared: false,
					page_size_log2: None,
				});
				encoder.section(&new_memory_section);
			}
			WasmSection::Raw { id, data } => {
				encoder.section(&wasm_encoder::RawSection { id, data: &data });
			}
		}
	}

	let wasm_bytes = encoder.finish();
	wasmparser::Validator::new().validate_all(&wasm_bytes)?;

	let output_dir = Path::new(output_dir);
	std::fs::create_dir_all(output_dir)?;

	let mut output_js = File::create(output_dir.join("docfind.js"))?;
	output_js.write_all(docfind_js)?;

	let mut output_wasm = File::create(output_dir.join("docfind_bg.wasm"))?;
	output_wasm.write_all(&wasm_bytes)?;

	let duration = start.elapsed();
	println!("WASM creation completed in: {:?}", duration);

	Ok(())
}
