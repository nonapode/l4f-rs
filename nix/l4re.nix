{ stdenv, gnumake, perl, coreutils, flex, bison, ncurses, which, l4re-mk-src, l4re-core-src }:
{ target, config, crossCompile ? "", crossGcc ? null }:

stdenv.mkDerivation {
  pname = "l4re-${target}";
  version = l4re-core-src.lastModifiedDate or "latest";
  srcs = [ l4re-mk-src l4re-core-src ];
  outputs = [ "out" "dev" ];

  nativeBuildInputs = [ gnumake perl coreutils flex bison ncurses which ]
    ++ (if crossGcc != null then [ crossGcc ] else [ ]);

  unpackPhase = ''
    cp -r ${l4re-mk-src} l4
    chmod -R +w l4
    mkdir -p l4/pkg/l4re-core
    cp -rt l4/pkg/l4re-core ${l4re-core-src}/*
    chmod -R +w l4/pkg
  '';

  postPatch = "patchShebangs .";
  enableParallelBuilding = true;
  CROSS_COMPILE = crossCompile;

  configurePhase = ''
    rm -rf build
    cat l4/mk/defconfig/config.${config} >$TMPDIR/config
    unset AR AS CC CXX LD
    make -C l4 B=$(pwd)/build DEFCONFIG=$TMPDIR/config
    make -C build olddefconfig
  '';

  makeFlags = [ "-C" "build" ];
  preBuild = "unset AR AS CC CXX LD; makeFlagsArray+=(\"L4DIR=$PWD/l4\")";
  postBuild = "make -C build sysroot";

  installPhase = ''
    mkdir -p $out/bin $dev/sysroot $dev/share/l4re

    if [ -d "build/bin" ]; then
      cp -rt $out/bin build/bin/*
      # 🎯 KORREKTUR: Wir entfernen '-perm -0100', um unmarkierte Binaries zu finden,
      # und erzwingen die korrekten Namen rekursiv aus den Arch-Ordnern heraus.
      find build/bin -type f -not -path "*/host/*" \( -name "moe" -o -name "ned" -o -name "sigma0" \) | while read -r bin; do
        echo "Found embedded target component: $bin"
        cp -v "$bin" $out/bin/
        chmod +x $out/bin/$(basename "$bin")
      done
    fi

    if [ -d "build/sysroot" ]; then
      cp -r build/sysroot/* $dev/sysroot/
    fi

    cp -r l4 $dev/share/l4re/source
    cp -r build $dev/share/l4re/build
  '';
}
