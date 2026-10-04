{
  description = "QuickMemos - a native Wayland GUI client for Memos";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            cargo
            rustc
            rust-analyzer
            pkg-config
            gtk4
            libadwaita
            glib
            pango
            cairo
            gdk-pixbuf
            graphene
            gobject-introspection
            dbus
          ];

          LD_LIBRARY_PATH = with pkgs;
            lib.makeLibraryPath [ gtk4 libadwaita glib pango cairo gdk-pixbuf graphene ];
        };

        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "quickmemos";
          version = "0.1.0";
          src = self;
          cargoLock.lockFile = ./Cargo.lock;
          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = with pkgs; [
            gtk4
            libadwaita
            glib
            pango
            cairo
            gdk-pixbuf
            graphene
          ];
          meta.mainProgram = "quickmemos";
        };
      });
}