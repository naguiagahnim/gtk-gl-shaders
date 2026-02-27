{
  description = "Rust dev flake";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };
  outputs = {
    self,
    nixpkgs,
  }: let
    pkgs = nixpkgs.legacyPackages."x86_64-linux";
  in {
    devShells."x86_64-linux".default = pkgs.mkShell {
      buildInputs = with pkgs; [
        cargo
        gtk4
        ninja
        gjs
        libepoxy
        cairo
        pango
        gdk-pixbuf
        glib
        pkg-config
        meson
        gobject-introspection
        rustc
        rustfmt
        clippy
        rust-analyzer
      ];
      env.RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";

      shellHook = ''
        PANGO_TYPELIB=$(find /nix/store -name "PangoCairo-1.0.typelib" 2>/dev/null | head -1 | xargs dirname)
        HARFBUZZ_TYPELIB=$(find /nix/store -name "HarfBuzz-0.0.typelib" 2>/dev/null | head -1 | xargs dirname)
        export GI_TYPELIB_PATH=${pkgs.gtk4}/lib/girepository-1.0:${pkgs.glib}/lib/girepository-1.0:${pkgs.graphene}/lib/girepository-1.0:${pkgs.gobject-introspection}/lib/girepository-1.0:${pkgs.gdk-pixbuf}/lib/girepository-1.0:$PANGO_TYPELIB:$HARFBUZZ_TYPELIB
      '';
    };
  };
}
