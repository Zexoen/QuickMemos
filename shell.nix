{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
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
    openssl
    perl
    dbus
  ];

  LD_LIBRARY_PATH = with pkgs; lib.makeLibraryPath [
    gtk4
    libadwaita
    glib
    pango
    cairo
    gdk-pixbuf
    graphene
    openssl
  ];
}