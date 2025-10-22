# String Analyzer Service

A RESTful API service built with Rust and Axum that analyzes strings and stores their computed properties.

## Features

- Analyze strings and compute multiple properties: length, palindrome check, unique characters, word count, SHA-256 hash, character frequency map
- Store analyzed strings in memory with timestamps
- Filter and search strings with query parameters
- Natural language filtering support
- RESTful API endpoints

## Requirements

- Rust 2024 or later
- Cargo package manager

## Installation

1. Clone the repository:

```bash
git clone <repository-url>
cd string-analyzer-app
```

2. Install dependencies:

```bash
cargo build
```

## Running Locally

1. Build and run the application:

```bash
cargo run
```

The server will start on `http://localhost:3000`

Deployed Link: https://string-analyzer-app-idowuseyi5141-r5fzwkr0.leapcell.dev/

For production release:

```bash
cargo build --release
./target/release/string-analyzer-app
```

## API Endpoints

### Create/Analyze String

**POST** `/strings`

Request:

```json
{
  "value": "string to analyze"
}
```

Success Response (201 Created):

```json
{
  "id": "sha256_hash_value",
  "value": "string to analyze",
  "properties": {
    "length": 17,
    "is_palindrome": false,
    "unique_characters": 12,
    "word_count": 3,
    "sha256_hash": "abc123...",
    "character_frequency_map": {
      "s": 2,
      "t": 3,
      "r": 2
    }
  },
  "created_at": "2025-08-27T10:00:00Z"
}
```

Error: 409 Conflict if string exists, 400 Bad Request if invalid

### Get Specific String

**GET** `/strings/{string_value}`

Returns: String data or 404 Not Found

### Get All Strings with Filtering

**GET** `/strings`

Query Parameters:

- `is_palindrome`: boolean (true/false)
- `min_length`: integer
- `max_length`: integer
- `word_count`: integer
- `contains_character`: single character

Response:

```json
{
  "data": [
    /* array of strings */
  ],
  "count": 15,
  "filters_applied": {
    /* applied filters */
  }
}
```

### Natural Language Filtering

**GET** `/strings/filter-by-natural-language`

Query parameter: `query` (URL encoded natural language query)

Examples:

- `query=all%20single%20word%20palindromic%20strings`
- `query=strings%20longer%20than%2010%20characters`
- `query=palindromic%20strings%20that%20contain%20the%20first%20vowel`
- `query=strings%20containing%20the%20letter%20z`

Response:

```json
{
  "data": [
    /* matching strings */
  ],
  "count": 3,
  "interpreted_query": {
    "original": "query text",
    "parsed_filters": {
      /* filters */
    }
  }
}
```

### Delete String

**DELETE** `/strings/{string_value}`

Success: 204 No Content, Error: 404 Not Found

## Dependencies

The project uses the following main dependencies (see `Cargo.toml` for full list):

- `axum`: Web framework
- `tokio`: Async runtime
- `serde`: Serialization/deserialization
- `serde_json`: JSON handling
- `sha2`: SHA-256 hashing
- `chrono`: Date/time handling

## Usage Examples

### Analyze a string

```bash
curl -X POST http://localhost:8080/strings \
  -H "Content-Type: application/json" \
  -d '{"value": "hello world"}'
```

### Get palindromes

```bash
curl "http://localhost:3000/strings?is_palindrome=true"
```

### Natural language query

```bash
curl "http://localhost:3000/strings/filter-by-natural-language?query=all%20single%20word%20palindromic%20strings"
```

## Project Structure

```
src/
  main.rs    - Application entry point with Axum routes and handlers
Cargo.toml   - Dependencies and project configuration
README.md    - This file
```

## Notes

- Strings are identified uniquely by SHA-256 hash
- In-memory storage (data resets on restart)
- Automatic URL decoding handles spaces and special characters
- All text comparisons are case-insensitive

### How To Deploy Rust on leapcell.io

1. Create a new Rust project or use an existing one.
2. Ensure your project has a `Cargo.toml` file with the necessary dependencies.
3. Commit your code to a Git repository.
4. Log in to your leapcell.io account.
5. Create a new application and select Rust as the runtime.
6. Connect your Git repository to the leapcell.io application.
7. Configure build and start commands:
   - Build Command: `cargo build --release`
   - Start Command: `./target/release/your_project_name`
     specify the port you use in the project or set/create the environment variable on leapcell if you're using the environment variable
8. Deploy the application and monitor the build logs for any errors.
   --- a/file:///home/oluwaseyi/dev247/rust/string-analyzer-app/src/main.rs
   +++ b/file:///home/oluwaseyi/dev247/rust/string-analyzer-app/src

use the given link under the domain to access your app or add your domain name.
