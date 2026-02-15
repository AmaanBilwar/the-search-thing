<h1 align="center">the-search-thing</h1>

<div align="center">
  <img src="branding/logo-white-bg.webp" alt="the-search-thing" width="400" />
  <p>Semantically search for your files, instantly.</p>
</div>

## What it is

the-search-thing is a local-first search system that makes your files, images, and videos
semantically searchable from one place.

## Features

- Semantic search across files, images, and videos
- Sub-millisecond response targets for interactive search
- Directory indexing with ignore rules
- Desktop UI with file open actions
- Natural language queries with ranked results

## Architecture (high level)

- Rust + PyO3 (`src/`): filesystem walking, video chunking, audio extraction, thumbnail capture
- Python indexers (`backend/indexer/`): file embeddings, video transcript + frame summary embeddings, image summary embeddings
- Helix DB (`db/schema.hx`, `db/queries.hx`): graph + vector storage
- FastAPI (`backend/app.py`): indexing/search API
- Electron UI (`client/`): desktop search experience
- Directory indexing with ignore rules
- Desktop UI with file open actions

## UI flow

<div align="center">
  <img src="docs/demo.gif" alt="Search demo" width="800" />
  <p>Demo video or GIF (coming soon)</p>
</div>

- Choose a folder to index
- Enter a natural language query
- Open results directly from the app

## Contributing

See `CONTRIBUTING.md` for setup, dev workflow, and frontend website instructions.

## Try it without dev setup

Download the Windows `.exe` release from GitHub Releases (coming soon).

## Release

We will ship a Windows `.exe` release so users can try it without a dev setup.

## Technologies used

- Rust + PyO3 for fast local indexing primitives
- Python for orchestration and API services
- FastAPI for the HTTP layer
- Helix DB for vector + graph storage
- Groq for transcription and vision summaries
- Electron + React for the desktop app

## License

GPL-3.0-only. See `LICENSE` for details.
