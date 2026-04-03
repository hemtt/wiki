# HEMTT Wiki - Website Generation

This document explains how the HEMTT Wiki website is built and deployed using Angular and Rust.

## Quick Start

Build and generate the complete website:

```bash
# Build Angular frontend
cd website && npm install && npm run build:prod && cd ..

# Generate metadata
cargo build --release -p arma3-wiki-website
./target/release/website

# Or use the build script
bash build-website.sh

# Output: dist-website/
```

## Architecture

```
┌─────────────────────────────────────────┐
│  dist-website/ (Final Output)           │
│  ├── index.html                         │
│  ├── main.[hash].js (Angular)           │
│  ├── styles.[hash].css (Tailwind)       │
│  └── assets/data/                       │
│      ├── commands.json (Rust-generated) │
│      └── filters.json (Rust-generated)  │
└─────────────────────────────────────────┘
         ↑                   ↑
         │                   │
    Angular Build        Rust Generator
    (website/)           (bin-website/)
         │                   │
    TypeScript         Command Parser
    Tailwind CSS       YAML Loader
```

## File Structure

```
.
├── website/                          # Angular application (modern frontend)
│   ├── src/
│   │   ├── app/
│   │   │   ├── components/
│   │   │   │   ├── command-viewer/  # Main page with filters
│   │   │   │   └── command-card/    # Individual command display
│   │   │   ├── services/
│   │   │   │   └── command.service.ts # Data service
│   │   │   └── app.routes.ts        # Routing
│   │   ├── main.ts                  # Bootstrap
│   │   ├── index.html               # HTML template
│   │   └── styles.css               # Global styles + Tailwind
│   ├── angular.json                 # Build config
│   ├── package.json                 # Dependencies
│   ├── tailwind.config.js           # Tailwind config
│   └── tsconfig.json                # TypeScript config
│
├── bin-website/                      # Rust metadata generator
│   ├── src/
│   │   ├── main.rs                  # Entry point
│   │   ├── models.rs                # Data structures
│   │   └── generator.rs             # Metadata generation
│   └── Cargo.toml
│
├── dist/                             # Command data (from bin-parse)
│   ├── commands/                    # YAML files per command
│   └── report.json                  # Parsing report
│
└── dist-website/                     # Generated website OUTPUT
    ├── index.html                   # Main page (from Angular)
    ├── main.[hash].js               # Application code (Angular)
    ├── styles.[hash].css            # Styling (Tailwind)
    └── assets/data/
        ├── commands.json            # All commands (Rust-generated)
        └── filters.json             # Filter options (Rust-generated)
```

## How It Works

### 1. Angular Frontend

**Location**: `website/`

- **Framework**: Angular 18+ with standalone components
- **Styling**: Tailwind CSS with PostCSS
- **Language**: TypeScript

**Components**:
- `CommandViewerComponent` - Main page with search/filter controls
- `CommandCardComponent` - Displays individual commands

**Service**:
- `CommandService` - Loads metadata from JSON files and provides filtering/search

**Build Output**: `dist-website/` (HTML, JS, CSS)

### 2. Rust Metadata Generator

**Location**: `bin-website/`

- **Input**: 
  - `dist/commands/*.yml` - Command definitions
  - `dist/report.json` - Command status
- **Output**:
  - `assets/data/commands.json` - Full command data
  - `assets/data/filters.json` - Filter options

**Data Flow**:
```
YAML files + report.json
         ↓
    Rust parser
         ↓
    JSON files (structured data)
         ↓
    Angular app loads at runtime
```

## Build Pipeline

### Development

```bash
# Terminal 1: Watch Angular development mode
cd website
npm start  # Serves at localhost:4200

# Terminal 2: Make changes to Rust if needed
cargo build -p arma3-wiki-website
./target/release/website
```

### Production

```bash
# One-command build
bash build-website.sh

# Or manual steps:
# 1. Build Angular
cd website
npm run build:prod
cd ..

# 2. Build and run Rust generator
cargo build -p arma3-wiki-website --release
./target/release/website
```

## Deployment

### GitHub Pages (Automated)

The `.github/workflows/deploy-website.yml` workflow:

1. **On push to main** (when relevant files change):
   - Installs Node.js dependencies
   - Builds Angular application
   - Compiles Rust metadata generator
   - Generates metadata JSON files
   - Uploads to GitHub Pages

2. **Setup**:
   - Go to repository Settings → Pages
   - Select "Deploy from a branch"
   - Choose `gh-pages` branch, `/ (root)` folder
   - Workflow will auto-create and deploy

### Self-Hosted

Serve the `dist-website/` directory with any web server:

```bash
# Python 3
python -m http.server --directory dist-website 8000

# Node.js (http-server)
npx http-server dist-website

# Docker
docker run -p 8080:80 -v $(pwd)/dist-website:/usr/share/nginx/html nginx

# Apache/Nginx - point DocumentRoot to dist-website/
```

## Customization

### Change UI/Layout

Edit Angular components in `website/src/app/`:
- Modify HTML templates (`.html` files)
- Update component logic (`.ts` files)
- Adjust Tailwind classes

### Change Styling

Edit `website/src/styles.css` and `website/tailwind.config.js`:
- Add custom colors
- Adjust responsive breakpoints
- Add custom utilities

### Add Features

Examples:
- Create new routes/pages
- Add filter by "since" version
- Show detailed command syntax
- Export to CSV/JSON
- API endpoint exposure

### Change Metadata Format

Modify `bin-website/src/generator.rs`:
- Update `generate_commands_file()` to change command data structure
- Update `generate_filters_file()` to change filter options
- Update `CommandService` in Angular to consume new format

## Dependencies

### Frontend
- Angular 18+
- TypeScript 5.5+
- Tailwind CSS 3.4+
- RxJS 7.8+

### Backend
- Rust 1.56+
- serde, serde_json, serde_yaml
- tokio, chrono, anyhow

## Performance

- **Build time**: ~30 seconds (full build)
- **Output size**: ~200KB gzipped
- **Runtime**: Near-instant search/filter (client-side)
- **Load time**: < 1 second

## Troubleshooting

### Angular won't build
```bash
cd website
npm install  # Ensure dependencies installed
npm run build:prod
```

### Metadata files missing
```bash
./target/release/website
# Check that dist/ and dist/report.json exist
```

### Website shows empty commands
- Check browser console for network errors
- Verify `assets/data/commands.json` exists
- Check CORS if hosting on different domain

### Deploy fails on GitHub Actions
- Check Actions logs for specific error
- Verify Node.js version matches workflow
- Ensure `Cargo.lock` is committed

## Development Notes

- Angular uses strict mode TypeScript (no `any` types)
- Tailwind classes are compiled to CSS (no runtime)
- Metadata is loaded once at app startup
- All filtering/search happens client-side (fast, no server needed)
- Responsive design handles mobile to desktop

## See Also

- [bin-website/README.md](bin-website/README.md) - Detailed generator documentation
- [website/README.md](website/README.md) - Angular app documentation
- [Angular Documentation](https://angular.io)
- [Tailwind CSS Documentation](https://tailwindcss.com)
