@new-migration description:
	sqlx migrate add -s {{description}}

[working-directory: 'audio']
@install-ffmpeg:
	cargo vcpkg build

[working-directory: 'www']
@_build-www:
	pnpm run build

@build: _build-www
	cargo build

@build-release: _build-www
	cargo build --release
