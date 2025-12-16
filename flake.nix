{
  description = "fuxlsx - Terminal Excel viewer with interactive TUI, search, formulas, and export";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        # Use the latest stable Rust toolchain
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "clippy" "rustfmt" ];
        };

        # Define the package
        fuxlsx = pkgs.rustPlatform.buildRustPackage rec {
          pname = "fuxlsx";
          version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package.version;

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };
          doCheck = false; # No tests yet

          meta = with pkgs.lib; {
            description = "Terminal Excel viewer with interactive TUI, search, formulas, and export";
            homepage = "https://github.com/bgreenwell/fuxlsx";
            license = licenses.mit;
            maintainers = [ ];
            platforms = platforms.all;
          };
        };

      in
      {
        # Default package
        packages.default = fuxlsx;
        packages.fuxlsx = fuxlsx;

        # Development shell
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Rust toolchain
            rustToolchain
            
            # Development tools
            cargo-watch
            cargo-edit
            cargo-audit
            cargo-deny
            cargo-outdated
            cargo-expand # For macro expansion debugging
            
            # Additional development tools
            git
            
            # LSP and formatting tools
            rust-analyzer

            # Debugging tools
            gdb
          ];

          # For better terminal support
          TERM = "xterm-256color";
          
          # Development shell hook
          shellHook = ''
            echo "Welcome to the fuxlsx Nix development environment!"
            echo ""
            echo "Dependencies loaded:"
            echo "  - Rust ${rustToolchain.version} with clippy, rustfmt, rust-src"
            echo "  - ratatui for terminal UI"
            echo "  - crossterm for cross-platform terminal"
            echo "  - arboard for clipboard support"
            echo "  - calamine for Excel parsing"
            echo ""
            echo "Nix commands:"
            echo "  nix build                - Build the project"
            echo "  nix run                  - Run fuxlsx"
            echo "  nix run . -- --help      - Run with help flag"
            echo "  nix develop              - Enter this dev shell"
            echo "  nix flake check          - Run all checks (fmt, clippy, build)"
            echo ""
            echo "Usage examples:"
            echo "  nix run . -- spreadsheet.xlsx"
            echo "  nix run . -- spreadsheet.xlsx --sheet Sales"
            echo "  nix run . -- spreadsheet.xlsx --search 'keyword'"
            echo "  nix run . -- spreadsheet.xlsx --export csv"
            echo ""
            echo "Development commands (if you need them):"
            echo "  cargo build              - Direct build (uses Nix env)"
            echo "  cargo watch -x run       - Live reload during development"
            echo "  cargo clippy             - Run linter"
            echo "  cargo fmt                - Format code"
            echo ""
            echo "Pro tip: 'nix run github:bgreenwell/fuxlsx -- file.xlsx' to run from anywhere!"
            echo ""
          '';
        };

        # Apps for easy running
        apps.default = {
          type = "app";
          program = "${fuxlsx}/bin/fuxlsx";
        };

        # Checks
        checks = {
          build = fuxlsx;
          
          # Add format check
          fmt-check = pkgs.runCommand "fmt-check" {
            buildInputs = [ rustToolchain ];
          } ''
            cd ${self}
            cargo fmt --all -- --check
            touch $out
          '';
        };
      });
}
