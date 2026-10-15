# The L4Re userland and image builder
{ stdenv, gnumake, perl, coreutils, libiconv, l4re-src, arch, config, pkgsBuildBuild, flex, bison, ncurses, writeText, which }:

stdenv.mkDerivation {
  pname = "l4re";
  version = l4re-src.version or "latest";
  src = l4re-src;
  outputs = [ "out" "host" "tools" ];

  # Host-Werkzeuge für den Build-Prozess
  nativeBuildInputs = [ gnumake perl coreutils libiconv pkgsBuildBuild.stdenv.cc flex bison ncurses which ];

  postPatch = ''
    # Behebt "Permission denied": Macht alle Build-Skripte ausführbar
    chmod -R +x mk/ tool/

    find tool mk -type f -exec sed -i 's|/usr/bin/env|env|g' {} +
    patchShebangs .
  '';

  enableParallelBuilding = true;

  # Wir löschen die blockierenden Nix-Umgebungsvariablen kurz vor dem Makefile-Aufruf
  configurePhase = ''
        runHook preConfigure

        # Verhindert den Fehler "'AR' specified in environment"
        unset AR AS LD CC CXX STRIP OBJCOPY OBJDUMP NM RANLIB

        # Wir erstellen die von L4Re gewünschte lokale Konfigurationsdatei
        cat << EOF > Makeconf.local
        HOSTCC       = ${pkgsBuildBuild.stdenv.cc}/bin/cc
        HOST_CC      = ${pkgsBuildBuild.stdenv.cc}/bin/cc
        HOSTCXX      = ${pkgsBuildBuild.stdenv.cc}/bin/g++
        HOST_CXX     = ${pkgsBuildBuild.stdenv.cc}/bin/g++
        CROSS_COMPILE = ${stdenv.cc}/bin/${stdenv.cc.targetPrefix}
    EOF

        # L4Re erzeugt hier den Build-Ordner 'build'
        make B=$(pwd)/build T=${config}

        runHook postConfigure
  '';

  # Für die Hauptphase (buildPhase) löschen wir die Umgebungsvariablen ebenfalls,
  # bevor Nix das Make-Kommando ausführt.
  preBuild = ''
    unset AR AS LD CC CXX STRIP OBJCOPY OBJDUMP NM RANLIB
  '';

  # Sichere Zuweisung über makeFlags
  makeFlags = [
    "-C"
    "build"
  ];

  installPhase = ''
    runHook preInstall

    mkdir -p $out $host
    # Hier folgt dein originales Image-Skript (z.B. make elfimage)
    find build/bin -maxdepth 1 -mindepth 1 -name host -o -print | while read d; do
      cp -rLt $out $d/*
    done
    cp -rL build/bin/host/* $host/
    cp -rL tool/ $tools

    runHook postInstall
  '';

  dontFixup = true;
}
