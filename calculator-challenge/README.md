# Calculator Challenge

A comprehensive calculator library implementation created through AI-driven evolution.

## Requirements

This project tests the orchestrator's ability to guide an AI agent to create:

- **Complex functionality**: Multiple features with interdependencies
- **Robust error handling**: Result types, edge cases, clear error messages
- **High code quality**: Documentation, best practices, design patterns
- **Comprehensive testing**: Edge cases, error conditions, >90% coverage

## Running the Evolution

```bash
cd ../orchestrator
cargo run -- --target ../calculator-challenge
```

## Monitoring Progress

```bash
cd ../dashboard
uv run tui.py --target ../calculator-challenge
```

## Expected Outcome

The AI agent should create:
1. A `Calculator` struct with builder pattern
2. All arithmetic operations with proper error handling
3. Memory functions (store/recall/clear)
4. Percentage calculations
5. Comprehensive test suite covering all edge cases
6. Full documentation with examples
