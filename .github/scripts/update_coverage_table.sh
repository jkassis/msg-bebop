#!/bin/bash

cat <<EOF > new_coverage_table.md
<!-- COVERAGE-TABLE-START -->
## Coverage

| Language   | Coverage |
|------------|----------|
| Rust       | ![](coverage-badges/Rust.svg) |
| Go         | ![](coverage-badges/Go.svg) |
| TypeScript | ![](coverage-badges/TS.svg) |
<!-- COVERAGE-TABLE-END -->
EOF

awk '/<!-- COVERAGE-TABLE-START -->/{flag=1; print; system("cat new_coverage_table.md"); next} /<!-- COVERAGE-TABLE-END -->/{flag=0; next} !flag' README.md > README.tmp
mv README.tmp README.md
