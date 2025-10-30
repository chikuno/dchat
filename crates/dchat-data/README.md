# dchat-data

Data storage directory for dchat persistent files.

## Contents

This crate contains:
- SQLite database files (*.db)
- User uploads and media
- Configuration data
- Cache files

## Usage

This is not a code crate - it's a designated location for runtime data files.
All database connections should point to files within this directory.

## .gitignore

All data files are gitignored to prevent committing sensitive data:
- *.db (database files)
- uploads/ (user media)
- cache/ (temporary files)
