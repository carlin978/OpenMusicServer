@new-migration description:
	sqlx migrate add -s {{description}}

[working-directory: 'audio']
@install-ffmpeg:
	cargo vcpkg build

[working-directory: 'www']
@setup-www:
	pnpm install

@build:
	cd www && pnpm run build-dev
	cargo build

@build-release:
	cd www && pnpm run build
	cargo build --release
