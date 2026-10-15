{ fetchFromGitHub, stdenv }:
let
  hash = {
    "r-2026-W41" = {
      mk = "sha256-urY9v9Rrj1hVSEkNw3h9NdyXmbdNu11dm4woKZFRsiU=";
      core = "sha256-DXzKv01eGEc86EYQUasUMhH40cKOx45TkGc8spBBTXk=";
      fiasco = "sha256-aULMPTGLziZOuYq009/edmsIUctyHh67noQsMRnqHWY=";
      bootstrap = "sha256-ZAOe1jvcI6nnA8Z89eP0fTaqUFH2Di2b4Z/zCbHZKBA=";
      libfdt = "sha256-y5LeoZ/LRnV2GhXSfcdv8TKtqoN2gNXyalyQJpy6uMw=";
      drivers-frst = "sha256-s7f6MlWPnxWML5LC9rvB6rMalxpDxnNC0P/3arHVNIk=";
    };
  };
  version = "r-2026-W41";
in
rec {
  l4re-mk-src = fetchFromGitHub {
    owner = "L4Re";
    repo = "mk";
    rev = version; # Es wird empfohlen, dies auf einen festen Commit-Hash zu pinnen
    hash = hash.${version}.mk;
  };

  drivers-frst-src = fetchFromGitHub {
    owner = "L4Re";
    repo = "drivers-frst";
    rev = version; # Es wird empfohlen, dies auf einen festen Commit-Hash zu pinnen
    hash = hash.${version}.drivers-frst;
  };

  bootstrap-src = fetchFromGitHub {
    owner = "L4Re";
    repo = "bootstrap";
    rev = version; # Es wird empfohlen, dies auf einen festen Commit-Hash zu pinnen
    hash = hash.${version}.bootstrap;
  };

  libfdt-src = fetchFromGitHub {
    owner = "L4Re";
    repo = "libfdt";
    rev = version; # Es wird empfohlen, dies auf einen festen Commit-Hash zu pinnen
    hash = hash.${version}.libfdt;
  };


  l4re-core-src = fetchFromGitHub {
    owner = "L4Re";
    repo = "l4re-core";
    rev = version;
    hash = hash.${version}.core;
  };

  fiasco-src = fetchFromGitHub {
    owner = "L4Re";
    repo = "fiasco";
    rev = version;
    hash = hash.${version}.fiasco;
  };

  l4re-src = stdenv.mkDerivation {
    pname = "l4re-source";
    version = l4re-core-src.rev or "latest";

    dontUnpack = true;
    dontBuild = true;

    installPhase = ''
      mkdir -p $out
      cp -r ${l4re-mk-src}/* $out
      chmod -R +w $out
      mkdir -p $out/pkg/drivers-frst
      cp -r ${drivers-frst-src}/* $out/pkg/drivers-frst/
      mkdir -p $out/pkg/libfdt
      cp -r ${libfdt-src}/* $out/pkg/libfdt/
      mkdir -p $out/pkg/bootstrap
      cp -r ${bootstrap-src}/* $out/pkg/bootstrap/
      mkdir -p $out/pkg/l4re-core
      cp -r ${l4re-core-src}/* $out/pkg/l4re-core/
      chmod -R +w $out
      patchShebangs $out
    '';
  };
}
