# Msg - Polyglot Bebop Message Library

A high-performance, cross-language message serialization library using Bebop.

## Overview

This project defines a single `Msg` schema using Bebop and generates serialization/deserialization libraries for all supported languages:

- **Rust** 🦀
- **C#** 🟣
- **TypeScript/JavaScript** 🟨
- **Go** 🐹
- **Python** 🐍
- **C++** ⚡

## Schema

```bebop
struct Msg {
  1 -> string body;      // The actual message payload
  2 -> string fromId;    // Sender ID
  3 -> string id;        // ID for idempotency
  4 -> string[] toIds;   // Array of recipient IDs
  5 -> string type;      // Type of the message
}
```

## Generated Libraries

Each language gets a packaged library that can be imported:

- `rust/` → Cargo crate
- `csharp/` → NuGet package  
- `typescript/` → Yarn package
- `go/` → Go module
- `python/` → PyPI package
- `cpp/` → Header-only library

## Project Structure

```
msg/
├── schema/              # Bebop schema definitions
│   └── msg.bop         # The Msg struct definition
├── build.sh            # Build script for all languages
├── rust/               # Rust library
│   ├── src/lib.rs      # Library code with tests
│   ├── examples/       # Usage examples
│   └── Cargo.toml      # Rust package configuration
├── typescript/         # TypeScript/JavaScript library
│   ├── src/index.ts    # Library code with utilities
│   ├── test/           # Jest tests
│   ├── package.json    # Yarn package configuration
│   └── yarn.lock       # Yarn dependency lock file
├── go/                 # Go library
│   ├── utils.go        # Utility functions
│   ├── msg_test.go     # Go tests
│   └── go.mod          # Go module configuration
├── python/             # Python library
│   ├── msg/            # Python package
│   ├── tests/          # Python tests
│   └── setup.py        # PyPI package configuration
├── csharp/             # C# library
│   ├── tests/          # C# tests
│   └── msg.csproj      # NuGet package configuration
└── cpp/                # C++ library
    ├── examples/       # C++ examples
    └── CMakeLists.txt  # CMake build configuration
```

## Usage

### Rust
```rust
use bebop::Record;
use msg::Msg;

let msg = Msg {
    body: "Hello",
    from_id: "user1",
    id: "msg123",
    to_ids: vec!["user2"],
    _type: "chat",
};

let mut bytes = Vec::new();
msg.serialize(&mut bytes)?;
let decoded = Msg::deserialize(&bytes)?;
```

### TypeScript
```typescript
import { Msg } from 'msg';

const msg = Msg({
    body: "Hello",
    fromId: "user1",
    id: "msg123",
    toIds: ["user2"],
    type: "chat"
});

const bytes = msg.encode();
const decoded = Msg.decode(bytes);
```

### Go
```go
import "github.com/jkassis/msg-bebop/go"

msg := &bebopgen.Msg{
    Body:   "Hello",
    FromId: "user1",
    Id:     "msg123",
    ToIds:  []string{"user2"},
    Type:   "chat",
}

bytes, _ := msg.MarshalBebopTo(nil)
decoded := &bebopgen.Msg{}
decoded.UnmarshalBebop(bytes)
```

## Building

### Prerequisites

Before building, you'll need to install the required tools:

#### Bebop Compiler
```bash
# Install the main bebop compiler
yarn global add @bebop/compiler

# For Go support, also install bebopc-go
go install github.com/200sc/bebop/main/bebopc-go@latest
```

#### Language-Specific Tools
- **Rust**: Install via [rustup.rs](https://rustup.rs/)
- **Node.js/TypeScript**: Install [Node.js](https://nodejs.org/) and [Yarn](https://yarnpkg.com/)
- **Go**: Install from [golang.org](https://golang.org/dl/)
- **Python**: Python 3.7+ (optional, for Python library)
- **C#**: .NET SDK (optional, for C# library)
- **C++**: CMake and a C++17 compiler (optional, for C++ library)

### Build All Languages
```bash
./build.sh
```

Or build specific languages:
```bash
./build.sh rust      # Build only Rust
./build.sh go        # Build only Go
./build.sh typescript # Build only TypeScript
```

## Performance

Bebop provides excellent performance characteristics:
- **Speed**: 10-20x faster than JSON
- **Size**: 50-60% smaller than JSON
- **Zero-copy**: Support for zero-copy deserialization
- **Cross-platform**: Identical binary format across all languages
