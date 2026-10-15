# The Fiasco.OC microkernel for one target architecture.
{ stdenv, pkgsBuildBuild, gnumake, perl, coreutils, flex, bison, fiasco-src, arch, config }:
let
  host = tool: "${pkgsBuildBuild.gcc}/bin/${pkgsBuildBuild.gcc.targetPrefix}${tool}";

in
stdenv.mkDerivation {
  pname = "fiasco-${arch}";
  version = fiasco-src.rev or "latest";
  src = fiasco-src;
  outputs = [ "out" "dbg" ];

  # gnumake, perl, flex und bison laufen nativ auf dem Host (x86_64)
  nativeBuildInputs = [ gnumake perl coreutils flex bison ];

  postPatch = ''
    find tool -type f -exec sed -i 's|/usr/bin/env|env|g' {} +
    patchShebangs .
  '';

  enableParallelBuilding = true;

  # Das Kconfig-Tool (Kompilierung von 'conf' und 'fixdep') benötigt den
  # nativen Host-Compiler (\$CC_FOR_BUILD), welcher von Nix bereitgestellt wird.
  configurePhase = ''
    runHook preConfigure
    set -x
    make B=$(pwd)/build T=${config} \
      "HOSTCC=${host "gcc"}" \
      "HOSTCXX=${host "g++"}" \

    set +x
    runHook postConfigure
  '';


  # Da stdenv nun das echte Cross-Stdenv ist, liefert `stdenv.cc.targetPrefix`
  # das korrekte Präfix (z. B. aarch64-none-eabi-).
  makeFlags = [
    "-C"
    "build"
    "CROSS_COMPILE=${stdenv.cc.targetPrefix}"
    "HOSTCC=${host "gcc"}"
    "HOSTCXX=${host "g++"}"
    "HOST_CC=${host "gcc"}"
    "HOST_CXX=${host "g++"}"
  ];

  installPhase = ''
    runHook preInstall
    install -Dm755 build/fiasco $out/fiasco
    install -Dm755 build/fiasco.debug $dbg/fiasco
    runHook postInstall
  '';

  dontFixup = true;
}
