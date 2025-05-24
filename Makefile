.PHONY: all build install uninstall clean test deb run dev help

# Variabili
BINARY_NAME = stg-demo
INSTALL_PATH = /usr/local/bin
DOC_PATH = /usr/local/share/doc/standard-terminal-graphics

# Target di default
all: build

# Compila il progetto
build:
	@echo "🔨 Compilazione..."
	cargo build --release --bin $(BINARY_NAME)

# Compila e esegue
run:
	@echo "🚀 Compilazione ed esecuzione..."
	cargo run --bin $(BINARY_NAME)

# Modalità development con auto-reload
dev:
	@echo "🔄 Modalità sviluppo..."
	cargo watch -x "run --bin $(BINARY_NAME)"

# Esegue i test
test:
	@echo "🧪 Esecuzione test..."
	cargo test

# Crea pacchetto .deb
deb: build
	@echo "📦 Creazione pacchetto .deb..."
	cargo deb --no-build

# Installazione automatica
install:
	@echo "📥 Installazione..."
	@chmod +x install.sh
	@./install.sh

# Disinstallazione
uninstall:
	@echo "🗑️ Disinstallazione..."
	@chmod +x uninstall.sh
	@./uninstall.sh

# Installazione manuale (senza script)
install-manual: build
	@echo "📥 Installazione manuale..."
	sudo cp target/release/$(BINARY_NAME) $(INSTALL_PATH)/$(BINARY_NAME)
	sudo chmod +x $(INSTALL_PATH)/$(BINARY_NAME)
	sudo mkdir -p $(DOC_PATH)
	sudo cp README.md LICENSE $(DOC_PATH)/
	@echo "✅ Installazione completata! Esegui: $(BINARY_NAME)"

# Disinstallazione manuale
uninstall-manual:
	@echo "🗑️ Disinstallazione manuale..."
	sudo rm -f $(INSTALL_PATH)/$(BINARY_NAME)
	sudo rm -rf $(DOC_PATH)
	@echo "✅ Disinstallazione completata!"

# Pulizia file temporanei
clean:
	@echo "🧹 Pulizia..."
	cargo clean
	rm -rf target/debian/*.deb

# Verifica dipendenze
deps:
	@echo "🔍 Verifica dipendenze..."
	@which cargo > /dev/null || (echo "❌ Rust/Cargo non installato! Visita: https://rustup.rs/" && exit 1)
	@echo "✅ Rust: $$(cargo --version)"
	@which cargo-deb > /dev/null || (echo "⚠️ cargo-deb non installato. Installazione..." && cargo install cargo-deb)
	@echo "✅ cargo-deb disponibile"

# Formato del codice
fmt:
	@echo "✨ Formattazione codice..."
	cargo fmt

# Controllo linting
lint:
	@echo "🔍 Controllo linting..."
	cargo clippy -- -D warnings

# Release completa
release: clean fmt lint test build deb
	@echo "🎉 Release completata!"
	@echo "📦 Pacchetto .deb creato in: target/debian/"

# Aiuto
help:
	@echo "Standard Terminal Graphics - Makefile"
	@echo "====================================="
	@echo ""
	@echo "Target disponibili:"
	@echo "  build         - Compila il progetto"
	@echo "  run           - Compila ed esegue"
	@echo "  dev           - Modalità sviluppo con auto-reload"
	@echo "  test          - Esegue i test"
	@echo "  deb           - Crea pacchetto .deb"
	@echo "  install       - Installazione automatica"
	@echo "  uninstall     - Disinstallazione"
	@echo "  clean         - Pulizia file temporanei"
	@echo "  deps          - Verifica dipendenze"
	@echo "  fmt           - Formatta il codice"
	@echo "  lint          - Controllo linting"
	@echo "  release       - Release completa"
	@echo "  help          - Mostra questo aiuto"
	@echo ""
	@echo "Esempi:"
	@echo "  make install  - Installa STG Demo"
	@echo "  make run      - Esegue la demo"
	@echo "  make deb      - Crea pacchetto .deb"
