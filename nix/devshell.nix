{ mkShell, git, gnumake, perl, gcc, binutils, qemu, devRust, l4Sources, amd64CrossGcc, arm64CrossGcc, writeScriptBin, flex, bison, ncurses, pkg-config, writeTextFile }:
{ cargoTarget ? null, targetArch }:

let
  crossPrefix = if targetArch == "aarch64" then "aarch64-unknown-linux-gnu-" else "";

  l4reConfigScript = writeTextFile {
    name = "hello.cfg";
    text = ''
      -- Minimales L4Re Initialisierungsskript
      local L4 = require("L4");
      L4.default_loader:start({ log = L4.Env.log }, "rom/hello-world");
    '';
    destination = "/hello.cfg";
  };

  l4-test = writeScriptBin "l4-test" ''
    #!/usr/bin/env bash
    set -e
    set +x
    unset AR AS CC CXX LD

    CORES=$(nproc 2>/dev/null || echo 4)

    echo "🦀 Building all workspace crates..."
    cargo build --workspace --release

    echo "🛡️  Preparing local writeable L4Re/Fiasco build directories..."

    #if [ -d "build" ]; then
    #  find build -type l -exec chmod +w {} + 2>/dev/null || true
    #  chmod -R +w build 2>/dev/null || true
    #fi

    if [ ! -f "build/source/Makefile" ]; then
      echo "📁 Shadowing combined L4Re store source to writeable local workspace..."
      mkdir -p build/source
      cp -aT ${l4Sources.l4re-src} build/source
      chmod -R +w build/source
    fi
    export L4_SRC="$PWD/build/source"
    export L4DIR="''${L4_SRC}"

    # 1. Fiasco konfigurieren und bauen
    if [ ! -f "build/fiasco/Makefile" ]; then
      echo "⚙️  Configuring Fiasco microkernel..."
      make -C $FIASCO_SRC B=$PWD/build/fiasco T=${if targetArch == "aarch64" then "arm64-virt-el2" else "amd64-dfl"} CROSS_COMPILE="${crossPrefix}"
    fi
    make -C build/fiasco CROSS_COMPILE="${crossPrefix}" -j$CORES

    # 2. L4Re aus dem lokalen Schatten-Quellbaum bauen
    if [ ! -f "build/l4re/.config" ]; then
      echo "⚙️  Configuring L4Re Userland..."
      make -C $PWD/build/source B=$PWD/build/l4re DEFCONFIG=$PWD/build/source/mk/defconfig/config.${if targetArch == "aarch64" then "arm64-virt-v8a" else "amd64"} CROSS_COMPILE="${crossPrefix}"

      #if [ -d "build" ]; then chmod -R +w build 2>/dev/null || true; fi
      make -C build/l4re CROSS_COMPILE="${crossPrefix}" olddefconfig
    fi

    #if [ -d "build" ]; then chmod -R +w build 2>/dev/null || true; fi
    make -C build/l4re CROSS_COMPILE="${crossPrefix}" -j$CORES sysroot

    # 🎯 KORREKTUR: Wir bauen das bootstrap-Paket explizit im Objektbaum vor,
    # damit der Image-Builder das erwartete Verzeichnis vorfindet!
    echo "⚙️  Building bootstrap target structures..."
    make -C build/l4re CROSS_COMPILE="${crossPrefix}" -j$CORES

    echo "📝 Generating modules.list..."
    mkdir -p build/l4re
    cat << 'EOF' > build/l4re/modules.list
    entry hello-world
    kernel fiasco
    roottask moe rom/hello.cfg
    module ned
    module hello.cfg
    module hello-world
    module l4re
    EOF

    # Boot-Konfigurationsskript (ned.lua), das Moe mitteilt, was zu tun ist
    L4RE_STORE_BIN=$(nix build .#l4re.${if targetArch == "aarch64" then "arm64-virt-v8a" else "amd64"} --no-link --print-out-paths)/bin
    FIASCO_STORE_BIN=$(nix build .#fiasco.${if targetArch == "aarch64" then "arm64-virt-el2" else "amd64-dfl"} --no-link --print-out-paths)

    echo "📦 Packaging monolith bootstrap image via BID..."
    make -C build/l4re \
      CROSS_COMPILE="${crossPrefix}" \
      MODULES_LIST=$PWD/build/l4re/modules.list \
      MODULE_SEARCH_PATH="$PWD/target/${cargoTarget}/debug:$PWD/target/${cargoTarget}/release:hello-world:${l4reConfigScript}:$L4RE_STORE_BIN:$FIASCO_STORE_BIN" \
      E="hello-world" \
      -j$CORES \
      elfimage

    echo "✅ Success! Run 'nix run .#run.${targetArch}' to launch the system."
  '';
in
mkShell {
  packages = [
    git
    gnumake
    perl
    gcc
    binutils
    qemu
    flex
    bison
    ncurses
    pkg-config
    devRust
    amd64CrossGcc
    arm64CrossGcc
    l4-test
  ];
  RUST_SRC_PATH = "${devRust}/lib/rustlib/src/rust/library";
  CARGO_BUILD_TARGET = if cargoTarget != null then cargoTarget else "";
  shellHook = ''
    export L4_SRC="${l4Sources.l4re-src}"
    export FIASCO_SRC="${l4Sources.fiasco-src}"
    export L4DIR="$L4_SRC"
    export KCFLAGS_x86_64="-target x86_64-unknown-none"
    export KCFLAGS_aarch64="-target aarch64-unknown-none"
    echo "======================================================="
    echo "🛡️  Welcome to the L4Re & Fiasco Rust Development Shell 🛡️"
    echo "======================================================="
  '';
}

