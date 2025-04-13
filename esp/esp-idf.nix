{ stdenv, lib, fetchgit }:

stdenv.mkDerivation rec {
  name = "esp-idf";
  version = "5.4.0";

  src = builtins.fetchGit {
    url = "https://github.com/espressif/esp-idf.git";
    ref = "v5.4";
    rev = "6897a7bf40e27cabe7f5ef60e71a7426f4047c00";
    submodules = true;
  };	

  patches = [ ./esp-idf.patch ];

  installPhase = ''
    mkdir -p $out/
    cp -r * $out
  '';  
}
