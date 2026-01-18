
{
  description = "ESP32 Rust development - system deps via Nix, toolchain via espup";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachSystem [ "x86_64-linux" "aarch64-darwin" "aarch64-linux" "x86_64-darwin" ] (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        devShells.default = pkgs.mkShell {
          name = "esp-rust";

          packages = with pkgs; [
            # Rust toolchain manager - espup installs on top of this
            rustup

            # Build deps for esp-idf-sys / bindgen
            pkg-config
            openssl
            cmake
            ninja
            git
            llvmPackages.libclang
            cacert

            # Python for ESP-IDF
            (python312.withPackages (ps: with ps; [
              pip
              virtualenv
              pyserial
            ]))

            # Serial
            picocom

            # Convenience
            just
            cargo-generate
            nil
            nixd
          ] ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
            udev
            libusb1
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            libiconv
          ];

          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
          SSL_CERT_FILE = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";

          shellHook = ''
            if [ -f "$HOME/export-esp.sh" ]; then
              source "$HOME/export-esp.sh"
            fi

            # Ensure git can use system credential helpers
            export GIT_TERMINAL_PROMPT=1
          '';
        };
      }
    );
}
