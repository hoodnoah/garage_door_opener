#!/usr/bin/env fish
# Source this file to set up the development environment: source env.fish

# Detect platform and set LIBCLANG_PATH
if test (uname) = Darwin
    if command -v brew > /dev/null 2>&1
        set -x LIBCLANG_PATH (brew --prefix llvm)/lib
        set -x PKG_CONFIG_PATH (brew --prefix openssl)/lib/pkgconfig
    else
        echo "Warning: Homebrew not found. Please install from https://brew.sh"
    end
else if test (uname) = Linux
    for version in 18 17 16 15 14
        if test -d /usr/lib/llvm-$version
            set -x LIBCLANG_PATH /usr/lib/llvm-$version/lib
            break
        end
    end
    if not set -q LIBCLANG_PATH
        echo "Warning: Could not locate LLVM."
    end
end

# Source ESP toolchain if available
if test -f $HOME/export-esp.fish
    source $HOME/export-esp.fish
else if test -f $HOME/export-esp.sh
    echo "Warning: ~/export-esp.fish not found, but ~/export-esp.sh exists."
    echo "Re-run espup install to regenerate it for fish."
else
    echo "Warning: ~/export-esp.fish not found. Run 'just setup' if you haven't already."
end

# Load .env file if it exists
if test -f main/.env
    for line in (grep -v '^#' main/.env | grep -v '^$')
        set parts (string split -m 1 '=' $line)
        if test (count $parts) -eq 2
            set -x $parts[1] $parts[2]
        end
    end
end

echo "Development environment loaded!"
echo "Run 'just check-deps' to verify your setup."
