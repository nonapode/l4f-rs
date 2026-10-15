# A rust l4re-application, built for a bare-metal target.
{ lib, rustPlatform, rustTarget }:
{ package, workspace ? ../. }:
let
  cargo_toml = base: builtins.fromTOML (builtins.readFile ("${base}/Cargo.toml"));
  package_info = cargo_toml "${workspace}/${package}";
  workspace_info = cargo_toml workspace;
in

rustPlatform.buildRustPackage {
  pname = package_info.package.name;
  version = package_info.package.version;

  # Only the Rust workspace: edits to the nix files do not rebuild the app.
  src = lib.fileset.toSource {
    root = workspace;
    fileset = lib.fileset.unions [
      ../Cargo.toml
      ../Cargo.lock
      (../. + "/${package}")
      ../l4-bitfield
      ../l4-ipc
      ../l4-ipc-core
      ../l4-sys
    ];
  };

  # Cargo.lock is used to vendor and build dependencies offline.
  cargoLock.lockFile = ../Cargo.lock;

  # 1. Behebt den rust-lld Fehler aus Schritt 1
  auditable = false;

  # 2. Zwingt Nix dazu, NUR das Bare-Metal-Target zu bauen und
  # verhindert das automatische Injizieren des Host-Targets.
  dontCargoBuildHook = true; # Wir übernehmen die Build-Phase manuell, um die Flags exakt zu kontrollieren

  buildPhase = ''
    runHook preBuild

    # Führt cargo build exakt mit deinem gewünschten Target aus
    RUSTFLAGS="-C relocation-model=static -C link-arg=-no-pie" cargo build \
      --jobs "$NIX_BUILD_CORES" \
      --release \
      --frozen \
      --target ${rustTarget} \
      --package ${package} \
      --config profile.release.debug=2

    runHook postBuild
  '';

  # Bare-Metal-Tests können nicht nativ auf dem Build-Host ausgeführt werden.
  doCheck = false;

  # Install the executable for the requested bare-metal target.
  installPhase = ''
    runHook preInstall
    install -Dm755 \
      target/${rustTarget}/release/${package} \
      "$out/bin/${package}"
    runHook postInstall
  '';

  # Freestanding executable: don't strip or patch it.
  dontFixup = true;

  meta.mainProgram = package;
}
