{ stdenv, gnumake, perl, coreutils, flex, bison, fiasco-src }:
{ target, config, crossCompile ? "", crossGcc ? null }:

stdenv.mkDerivation {
  pname = "fiasco-${target}";
  version = fiasco-src.rev or "latest";
  src = fiasco-src;
  outputs = [ "out" "dbg" ];

  # crossGcc wird hier in den Suchpfad der Sandbox injiziert
  nativeBuildInputs = [ gnumake perl coreutils flex bison ]
    ++ (if crossGcc != null then [ crossGcc ] else [ ]);

  postPatch = ''
    find tool -type f -exec sed -i 's|/usr/bin/env|env|g' {} +
    patchShebangs .
  '';

  enableParallelBuilding = true;
  CROSS_COMPILE = crossCompile;

  configurePhase = "make B=$(pwd)/build T=${config}";
  makeFlags = [ "-C" "build" ];

  installPhase = ''
    install -Dm755 build/fiasco $out/fiasco
    install -Dm755 build/fiasco.debug $dbg/fiasco
    install -Dm755 build/bootstrap.fiasco.bin $dbg/bootstrap.fiasco.bin
  '';
}
