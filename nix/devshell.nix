# Development environment: toolchains and the pinned sources for browsing.
# Images are built in the nix store (`nix build .#image-<arch>`), not here.
{ lib
, mkShell
, git
, gnumake
, perl
, gcc
, binutils
, qemu
, flex
, bison
, ncurses
, pkg-config
, rust
, crossGccs
, sources
, cargoTarget ? null
}:

mkShell ({
  packages = [ git gnumake perl gcc binutils qemu flex bison ncurses pkg-config rust ]
    ++ crossGccs;

  RUST_SRC_PATH = "${rust}/lib/rustlib/src/rust/library";

  shellHook = ''
    export L4_SRC="${sources.l4re-src}"
    export FIASCO_SRC="${sources.fiasco-src}"
    echo "L4Re & Fiasco Rust development shell"
    echo "  nix build .#image-<x86_64|aarch64>   build the bootable image"
    echo "  nix run   .#run-<x86_64|aarch64>     boot it in QEMU"
  '';
} // lib.optionalAttrs (cargoTarget != null) { CARGO_BUILD_TARGET = cargoTarget; })
