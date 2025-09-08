@new-migration description:
	sqlx migrate add -s {{description}}

[working-directory: 'audio']
@install-ffmpeg:
	cargo vcpkg build
