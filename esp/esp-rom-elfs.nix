{ stdenv, lib, fetchurl }:

stdenv.mkDerivation rec {
  pname = "esp-rom-elfs";
  version = "20240305";

  src = fetchurl {
    url = "https://github.com/espressif/esp-rom-elfs/releases/download/${version}/esp-rom-elfs-${version}.tar.gz";
    hash = "sha256-omYJtBVxDwFj14WFDHaXUnFwBAWcEpxHLpoMvVTgQiw=";
  };

  buildInputs = [ ];

  phases = [ "unpackPhase" "installPhase" ];

  sourceRoot = ".";

  installPhase = ''
    cp -r . $out
  '';
}
