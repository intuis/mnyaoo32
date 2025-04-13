{ stdenv, lib, cmake, ninja,  }:

stdenv.mkDerivation rec {
  pname = "esp-tools";
  version = "0.1";

  buildInputs = [ ];

  phases = [ "installPhase" ];

  installPhase = ''
    CMAKE_DIR=$out/tools/cmake/3.30.2/cmake-3.30.2-linux-x86_64
    mkdir -p $CMAKE_DIR
    ln -s $cmake/bin $CMAKE_DIR/
    ln -s $cmake/share $CMAKE_DIR/
    echo hi

    XTENSA_DIR=$out/tools/xtensa-esp-elf/esp-14.2.0_20241119/
    mkdir -p $XTENSA_DIR
    ln -s $esp32/.rustup/toolchains/esp/xtensa-esp-elf/esp-14.2.0_20240906/xtensa-esp-elf/bin/* $XTENSA_DIR/

    NINJA_DIR=$out/tools/ninja/1.12.1
    mkdir -p $NINJA_DIR
    ln -s $ninja/bin/ninja $NINJA_DIR/.
  '';
}
