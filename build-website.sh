#!/bin/bash
# Build script for HEMTT Wiki website

set -e

echo "🔨 Building HEMTT Wiki website..."
echo ""

# Step 1: Build Angular app
echo "📦 Building Angular application..."
cd website
npm run build:prod
cd ..
echo "✅ Angular build complete"
echo ""

# Step 2: Generate metadata
echo "📊 Generating command metadata..."
cargo build -p arma3-wiki-website --release
./target/release/website
echo "✅ Metadata generation complete"
echo ""

echo "🎉 Website build complete!"
echo "📁 Output directory: dist-website/"
ls -lh dist-website/
