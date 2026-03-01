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

    rustLib = pkgs.rustPlatform.buildRustPackage {
      pname = "glarea-gjs-lib";
      version = "0.1.0";
      src = ./.;
      cargoLock.lockFile = ./Cargo.lock;
      buildInputs = with pkgs; [gtk4 glib libepoxy];
      nativeBuildInputs = with pkgs; [pkg-config];
    };

    girepo = pkgs.stdenv.mkDerivation {
      pname = "glarea-gjs-lib";
      version = "0.1.0";
      src = ./.;
      nativeBuildInputs = with pkgs; [
        meson
        ninja
        pkg-config
        gobject-introspection
      ];
      buildInputs = with pkgs; [gtk4 glib libepoxy];
      propagatedBuildInputs = [rustLib];
      mesonFlags = ["-Dprebuilt_so=${rustLib}/lib/libgtkglshaders.so"];

      meta = with pkgs.lib; {
        description = "GTK4 GLArea widget with custom GLSL shaders, exposed to GJS/JavaScript via GObject Introspection ";
        homepage = "https://github.com/naguiagahnim/glarea-gjs";
        license = licenses.gpl3Only;
        platforms = platforms.linux;
        maintainers = [];
      };
    };
  in {
    packages."x86_64-linux".default = girepo;

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
        export GI_TYPELIB_PATH=${pkgs.gtk4}/lib/girepository-1.0:${pkgs.glib}/lib/girepository-1.0:${pkgs.graphene}/lib/girepository-1.0:${pkgs.gobject-introspection}/lib/girepository-1.0:${pkgs.gdk-pixbuf}/lib/girepository-1.0:${pkgs.pango.out}/lib/girepository-1.0:${pkgs.harfbuzz}/lib/girepository-1.0
      '';
    };
  };
}
