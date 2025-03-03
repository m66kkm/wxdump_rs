# WxDump_RS

WxDump_RS is a Rust port of [PyWxDump](https://github.com/xaoyaoo/PyWxDump), a tool for obtaining WeChat account information, decrypting databases, viewing chat history, and exporting chat logs.

## Features

- Get WeChat account information (nicknames, accounts, phones, emails, database keys)
- Decrypt WeChat databases
- View WeChat chat history
- Export chat logs as HTML, CSV, or JSON

## Project Structure

The project is organized into several modules:

- `wx_core`: Core functionality for WeChat data extraction and decryption
- `db`: Database handling for different WeChat databases
- `api`: Web API for viewing and exporting chat history
- `cli`: Command-line interface

## Usage

```bash
# Get WeChat information
wxdump_rs info

# Decrypt WeChat database
wxdump_rs decrypt -k <key> -i <db_path> -o <out_path>

# View chat history
wxdump_rs dbshow --merge_path <merge_path>

# Start UI
wxdump_rs ui
```

## Status

This is a work in progress. The following components have been implemented:

- [x] Project structure
- [ ] CLI interface
- [ ] Core functionality
- [ ] Database handling
- [ ] Web API

## Compilation Errors

There are several compilation errors that need to be fixed:

1. Missing commas in function calls and imports
2. CloseHandle is not in the right module
3. Type annotations needed for Router
4. Missing From implementations for error types
5. Ambiguous method call for HmacSha1::new_from_slice
6. Deprecated function chrono::NaiveDateTime::from_timestamp_opt
7. PathBuf doesn't implement Display

## Next Steps

1. Fix compilation errors
2. Implement core functionality
3. Implement database handling
4. Implement web API
5. Add tests
6. Add documentation

## License

This project is licensed under the same license as the original PyWxDump project.
