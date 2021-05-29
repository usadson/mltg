if (!(Get-Command mdbook -ea SilentlyContinue)) {
    cargo install mdbook
} 
