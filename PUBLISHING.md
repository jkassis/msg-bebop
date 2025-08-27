# Publishing Guide - Msg Bebop Libraries

This guide shows how to publish the generated libraries to their respective package managers.

## Prerequisites

1. Bebop compiler installed (`yarn global add @bebop/compiler`)
2. Go-specific compiler installed (`go install github.com/200sc/bebop/main/bebopc-go@latest`)
3. Build completed (`./build.sh`)
4. Package manager accounts setup

## 📦 Publishing to Package Managers

### 1. Rust (Cargo) 🦀

```bash
cd rust
# Login to crates.io
cargo login <your-api-token>

# Publish
cargo publish
```

**Package URL**: `https://crates.io/crates/msg`

**Usage in other projects**:
```toml
[dependencies]
msg = "0.1.0"
```

### 2. TypeScript/JavaScript (Yarn) 🟨

```bash
cd typescript
# Login to npm
npm login

# Build and publish
yarn build
yarn publish
```

**Package URL**: `https://www.npmjs.com/package/msg`

**Usage in other projects**:
```bash
yarn add msg
```

### 3. Go (Go Modules) 🐹

```bash
cd go
# Tag and push
git tag v0.1.0
git push origin v0.1.0
```

**Module URL**: `github.com/jkassis/msg-bebop/go`

**Usage in other projects**:
```bash
go get github.com/jkassis/msg-bebop/go
```

### 4. Python (PyPI) 🐍

```bash
cd python
# Install publishing tools
pip install build twine

# Build package
python -m build

# Upload to PyPI
twine upload dist/*
```

**Package URL**: `https://pypi.org/project/msg/`

**Usage in other projects**:
```bash
pip install msg
```

### 5. C# (NuGet) 🟣

```bash
cd csharp
# Login to NuGet
dotnet nuget add source https://api.nuget.org/v3/index.json -n nuget.org

# Pack and publish
dotnet pack -c Release
dotnet nuget push bin/Release/Msg.Bebop.0.1.0.nupkg -k <your-api-key> -s https://api.nuget.org/v3/index.json
```

**Package URL**: `https://www.nuget.org/packages/Msg.Bebop/`

**Usage in other projects**:
```xml
<PackageReference Include="Msg.Bebop" Version="0.1.0" />
```

### 6. C++ (vcpkg/Conan) ⚡

For C++, you can:

**Option A: vcpkg**
```bash
# Create vcpkg port
vcpkg create msg
```

**Option B: Conan**
```bash
# Create Conan package
conan create . msg/0.1.0@
```

**Option C: Header-only distribution**
- Simply distribute the `cpp/include/` directory
- Users include directly in their projects

## 🏷️ Version Management

Update versions consistently across all packages:

1. **Schema version** in `schema/msg.bop` (comment)
2. **Rust**: `Cargo.toml` → `version`
3. **TypeScript**: `package.json` → `version`  
4. **Go**: Git tags (`git tag v0.1.0`)
5. **Python**: `setup.py` → `version`
6. **C#**: `*.csproj` → `<Version>`
7. **C++**: `CMakeLists.txt` → `VERSION`

## 🤖 Automated Publishing (GitHub Actions)

Create `.github/workflows/publish.yml`:

```yaml
name: Publish Libraries

on:
  release:
    types: [published]

jobs:
  publish-rust:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Build and publish
        run: |
          ./build.sh rust
          cd rust
          cargo publish --token ${{ secrets.CARGO_TOKEN }}

  publish-typescript:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
        with:
          node-version: '18'
          registry-url: 'https://registry.npmjs.org'
      - name: Install Yarn
        run: npm install -g yarn
      - name: Build and publish
        run: |
          ./build.sh typescript
          cd typescript
          yarn publish
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}

  # Similar jobs for other languages...
```

## 📊 Package Status Dashboard

Track your packages across ecosystems:

| Language | Package Manager | Status | Downloads | Version |
|----------|----------------|--------|-----------|---------|
| Rust 🦀 | crates.io | [![Crates.io](https://img.shields.io/crates/v/msg.svg)](https://crates.io/crates/msg) | ![Downloads](https://img.shields.io/crates/d/msg.svg) | v0.1.0 |
| TypeScript 🟨 | npm | [![npm](https://img.shields.io/npm/v/msg.svg)](https://www.npmjs.com/package/msg) | ![Downloads](https://img.shields.io/npm/dm/msg.svg) | v0.1.0 |
| Go 🐹 | pkg.go.dev | [![Go Reference](https://pkg.go.dev/badge/github.com/youruser/msg/go.svg)](https://pkg.go.dev/github.com/youruser/msg/go) | - | v0.1.0 |
| Python 🐍 | PyPI | [![PyPI](https://img.shields.io/pypi/v/msg.svg)](https://pypi.org/project/msg/) | ![Downloads](https://img.shields.io/pypi/dm/msg.svg) | v0.1.0 |
| C# 🟣 | NuGet | [![NuGet](https://img.shields.io/nuget/v/Msg.Bebop.svg)](https://www.nuget.org/packages/Msg.Bebop/) | ![Downloads](https://img.shields.io/nuget/dt/Msg.Bebop.svg) | v0.1.0 |

## 🔄 Maintenance

1. **Schema Evolution**: Update `msg.bop` schema carefully
2. **Version Bumping**: Use semantic versioning (major.minor.patch)
3. **Breaking Changes**: Increment major version
4. **Cross-Language Testing**: Ensure serialized data works across all languages
5. **Documentation**: Keep README files updated in each package

## 📈 Success Metrics

Monitor your polyglot library success:

- **Download counts** across all package managers
- **GitHub stars** and issues
- **Performance benchmarks** vs alternatives
- **Adoption** in real projects
- **Community contributions**

Your Bebop-based message library will provide:
- **10-20x faster** serialization than JSON
- **50-60% smaller** binary size
- **Zero-copy** deserialization capabilities  
- **Consistent** API across all languages
