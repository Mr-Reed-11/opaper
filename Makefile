PREFIX   ?= /usr
DESTDIR  ?=
BINDIR    = $(PREFIX)/bin
DATADIR   = $(PREFIX)/share
ICONDIR   = $(DATADIR)/icons/hicolor
APPDIR    = $(DATADIR)/applications
PLUGINDIR = $(DATADIR)/plasma/wallpapers/org.opaper.wallpaper

ENGINE_BIN = engine/target/release/opaper-engine
APP_BIN    = app/src-tauri/target/release/opaper

.PHONY: all engine app kde-plugin install uninstall clean

all: engine app kde-plugin

engine:
	cargo build --release --manifest-path engine/Cargo.toml

app:
	cd app && npm install && npm run build
	cargo build --release --manifest-path app/src-tauri/Cargo.toml --features custom-protocol

kde-plugin:
	bash packaging/kde-wallpaper/build.sh

install: all
	install -Dm755 $(ENGINE_BIN) $(DESTDIR)$(BINDIR)/opaper-engine
	install -Dm755 $(APP_BIN)    $(DESTDIR)$(BINDIR)/opaper
	install -Dm755 packaging/opaper-launch $(DESTDIR)$(BINDIR)/opaper-launch

	install -Dm644 packaging/br.com.nortixcode.opaper.desktop \
		$(DESTDIR)$(APPDIR)/opaper.desktop

	install -Dm644 app/src-tauri/icons/32x32.png \
		$(DESTDIR)$(ICONDIR)/32x32/apps/opaper.png
	install -Dm644 app/src-tauri/icons/64x64.png \
		$(DESTDIR)$(ICONDIR)/64x64/apps/opaper.png
	install -Dm644 app/src-tauri/icons/128x128.png \
		$(DESTDIR)$(ICONDIR)/128x128/apps/opaper.png
	install -Dm644 app/src-tauri/icons/128x128@2x.png \
		$(DESTDIR)$(ICONDIR)/256x256/apps/opaper.png
	install -Dm644 app/src-tauri/icons/icon.png \
		$(DESTDIR)$(ICONDIR)/512x512/apps/opaper.png

	for dir in examples/*/; do \
		name=$$(basename "$$dir"); \
		install -Dm644 "$$dir/manifest.json" \
			"$(DESTDIR)$(DATADIR)/opaper/examples/$$name/manifest.json"; \
		install -Dm644 "$$dir/shader.frag" \
			"$(DESTDIR)$(DATADIR)/opaper/examples/$$name/shader.frag"; \
	done

	# KDE Plasma wallpaper plugin
	install -Dm644 packaging/kde-wallpaper/org.opaper.wallpaper/metadata.json \
		$(DESTDIR)$(PLUGINDIR)/metadata.json
	install -Dm644 packaging/kde-wallpaper/org.opaper.wallpaper/contents/config/main.xml \
		$(DESTDIR)$(PLUGINDIR)/contents/config/main.xml
	install -Dm644 packaging/kde-wallpaper/org.opaper.wallpaper/contents/ui/main.qml \
		$(DESTDIR)$(PLUGINDIR)/contents/ui/main.qml
	for qsb in packaging/kde-wallpaper/org.opaper.wallpaper/contents/shaders/*.qsb; do \
		install -Dm644 "$$qsb" \
			"$(DESTDIR)$(PLUGINDIR)/contents/shaders/$$(basename $$qsb)"; \
	done

uninstall:
	rm -f  $(DESTDIR)$(BINDIR)/opaper-engine
	rm -f  $(DESTDIR)$(BINDIR)/opaper
	rm -f  $(DESTDIR)$(BINDIR)/opaper-launch
	rm -f  $(DESTDIR)$(APPDIR)/opaper.desktop
	rm -f  $(DESTDIR)$(ICONDIR)/32x32/apps/opaper.png
	rm -f  $(DESTDIR)$(ICONDIR)/64x64/apps/opaper.png
	rm -f  $(DESTDIR)$(ICONDIR)/128x128/apps/opaper.png
	rm -f  $(DESTDIR)$(ICONDIR)/256x256/apps/opaper.png
	rm -f  $(DESTDIR)$(ICONDIR)/512x512/apps/opaper.png
	rm -rf $(DESTDIR)$(DATADIR)/opaper/
	rm -rf $(DESTDIR)$(PLUGINDIR)

clean:
	cargo clean --manifest-path engine/Cargo.toml
	cargo clean --manifest-path app/src-tauri/Cargo.toml
	rm -rf app/dist app/node_modules
