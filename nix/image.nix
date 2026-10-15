# The L4Re userland and image builder
{ callPackage, lib, stdenv, gnumake, perl, coreutils, libiconv, fiasco, app, l4re, arch, config, pkgsBuildBuild, flex, bison, ncurses, writeText, which }:
let
  cfg = writeText "cfg" ''
    -- Minimal L4Re init script
    local L4 = require("L4");
    L4.default_loader:start({ log = L4.Env.log }, "rom/init");
  '';
  modulesList =
    let
    in
    writeText "modules.list" ''
      entry nix
      kernel fiasco -serial_esc
      roottask moe rom/nix-image.cfg
      module l4re
      module ned
      module[fname=nix-image.cfg] ${cfg}
      module[fname=init] ${app.meta.mainProgram or app.meta.name or app.pname}
    '';
in
stdenv.mkDerivation {
  pname = "image-${arch}";
  version = l4re.version;
  dontUnpack = true;

  # Host-Werkzeuge für den Build-Prozess
  nativeBuildInputs = [ gnumake perl coreutils libiconv pkgsBuildBuild.stdenv.cc flex bison ncurses which ];

  enableParallelBuilding = true;

  configurePhase = ''
    mkdir rom
    cat <<EOF >rom/nix-image.cfg
      -- Minimal L4Re init script
      local L4 = require("L4");
      L4.default_loader:start({ log = L4.Env.log }, "rom/init");
    EOF
    cat <<EOF >modules.list
      entry nix
      kernel fiasco -serial_esc
      roottask moe rom/nix-image.cfg
      module l4re
      module ned
      module nix-image.cfg
      module[fname=init] ${lib.getExe app}
    EOF
  '';

  buildPhase = ''
    ${l4re.tools}/bin/l4image -i ${l4re}/plain/bootstrap.elf -o bootstrap create --entry nix --search-path $(pwd)/rom:${app}/bin:${l4re}/l4f:${fiasco} --modules-list-file $(pwd)/modules.list
  '';

  installPhase = ''
    runHook preInstall

    mv bootstrap $out

    runHook postInstall
  '';

  dontFixup = true;
}
