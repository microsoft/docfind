#!/usr/bin/env python3
"""
Download and prepare the AG News dataset for the docfind demo.
AG News is a collection of news articles with titles and descriptions.
"""

import json
import urllib.request
import csv
import sys
import os

# AG News dataset URLs
# Primary: from the original source (GitHub raw content)
# Fallback: from HuggingFace
TRAIN_URLS = [
    "https://raw.githubusercontent.com/mhjabreel/CharCnn_Keras/master/data/ag_news_csv/train.csv",
    "https://huggingface.co/datasets/fancyzhx/ag_news/resolve/main/train.csv"
]
TEST_URLS = [
    "https://raw.githubusercontent.com/mhjabreel/CharCnn_Keras/master/data/ag_news_csv/test.csv",
    "https://huggingface.co/datasets/fancyzhx/ag_news/resolve/main/test.csv"
]

# Category mapping
CATEGORIES = {
    "1": "World",
    "2": "Sports",
    "3": "Business",
    "4": "Sci/Tech"
}

def download_file(url_list, output_path):
    """Download a file from a list of URLs (tries each in order)."""
    for url in url_list:
        print(f"Trying to download from {url}...")
        try:
            with urllib.request.urlopen(url, timeout=30) as response:
                data = response.read()
            with open(output_path, 'wb') as f:
                f.write(data)
            print(f"Successfully downloaded to {output_path} ({len(data)} bytes)")
            return True
        except Exception as e:
            print(f"Failed: {e}")
            continue
    
    print(f"Error: Could not download from any URL", file=sys.stderr)
    return False

def parse_csv_file(file_path, limit=None):
    """Parse AG News CSV file and convert to documents."""
    documents = []
    
    print(f"Parsing {file_path}...")
    
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            # AG News CSV format: "Class Index","Title","Description"
            reader = csv.reader(f)
            
            for i, row in enumerate(reader):
                if limit and i >= limit:
                    break
                    
                if len(row) < 3:
                    continue
                
                class_index = row[0].strip('"')
                title = row[1].strip('"')
                description = row[2].strip('"')
                
                category = CATEGORIES.get(class_index, "Unknown")
                
                # Create a unique href based on category and index
                href = f"/article/{category.lower().replace('/', '-')}/{i}"
                
                documents.append({
                    "title": title,
                    "category": category,
                    "href": href,
                    "body": description
                })
        
        print(f"Parsed {len(documents)} documents from {file_path}")
        return documents
        
    except Exception as e:
        print(f"Error parsing {file_path}: {e}", file=sys.stderr)
        return []

def main():
    # Configuration
    output_dir = os.path.dirname(os.path.abspath(__file__))
    train_csv = os.path.join(output_dir, "train.csv")
    test_csv = os.path.join(output_dir, "test.csv")
    output_json = os.path.join(output_dir, "documents.json")
    
    # By default, use all training data (120,000 articles)
    # Can be limited for testing
    limit = None
    if len(sys.argv) > 1:
        try:
            limit = int(sys.argv[1])
            print(f"Limiting to {limit} documents")
        except ValueError:
            print(f"Invalid limit: {sys.argv[1]}", file=sys.stderr)
            sys.exit(1)
    
    # Download datasets if they don't exist
    if not os.path.exists(train_csv):
        if not download_file(TRAIN_URLS, train_csv):
            print("Failed to download training data", file=sys.stderr)
            sys.exit(1)
    else:
        print(f"Using existing {train_csv}")
    
    # Parse the training data (the larger dataset)
    documents = parse_csv_file(train_csv, limit)
    
    if not documents:
        print("No documents parsed!", file=sys.stderr)
        sys.exit(1)
    
    # Write to JSON
    print(f"Writing {len(documents)} documents to {output_json}...")
    with open(output_json, 'w', encoding='utf-8') as f:
        json.dump(documents, f, ensure_ascii=False, indent=2)
    
    print(f"Successfully created {output_json}")
    print(f"Total documents: {len(documents)}")
    
    # Calculate and display statistics
    total_chars = sum(len(doc['title']) + len(doc['body']) for doc in documents)
    print(f"Total characters: {total_chars:,}")
    print(f"Average document size: {total_chars // len(documents):,} characters")
    
    # Count by category
    category_counts = {}
    for doc in documents:
        cat = doc.get('category', 'Unknown')
        category_counts[cat] = category_counts.get(cat, 0) + 1
    
    print("\nDocuments by category:")
    for cat, count in sorted(category_counts.items()):
        print(f"  {cat}: {count:,}")

if __name__ == "__main__":
    main()
