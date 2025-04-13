{
  description = "Flake for esp32";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-24.11";
    esp32 = {
      url = "github:micielski/esp32";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self , nixpkgs, esp32, ... }: let
    idf-rust = esp32.packages.x86_64-linux.esp32;
    system = "x86_64-linux";
  in {
    devShells."${system}".default = let
      pkgs = import nixpkgs {
        inherit system;
      };
      esp-tools = pkgs.callPackage ./esp/esp-tools.nix {};
      esp32ulp = pkgs.callPackage ./esp/esp32ulp-elf.nix {};
      esp-idf = pkgs.callPackage ./esp/esp-idf.nix {};
      cmake = pkgs.cmake;
      ninja = pkgs.ninja;
    in pkgs.mkShell {
      name = "esp-idf-env";
      buildInputs = with pkgs; [

        git
        wget
        gnumake
        esp-idf

        flex
        bison
        gperf
        pkg-config

        cmake

        ncurses5

        ninja
      ];
      packages = with pkgs; [
        (pkgs.callPackage ./esp/esp-toolchain.nix {})
        (pkgs.callPackage ./esp/esp-clang-toolchain.nix {})
        esp32ulp
        esp-tools
        idf-rust
        rust-analyzer
        espflash
        ldproxy
        python312
        cmake
        ninja
        flex
        bison
        git
        wget
        gnumake
        pkg-config
        ncurses5
        clang
      ];

      shellHook = ''
        export IDF_PATH=$(pwd)/esp-idf
        export PATH=$IDF_PATH/tools:$PATH
        export PATH="${idf-rust}/.rustup/toolchains/esp/bin:$PATH"
        export RUST_SRC_PATH="$(rustc --print sysroot)/lib/rustlib/src/rust/src"
        export LIBCLANG_PATH="${idf-rust}/.rustup/toolchains/esp/xtensa-esp32-elf-clang/esp-18.1.2_20240912/esp-clang/lib/"
        # export ESP_IDF_TOOLS_INSTALL_DIR=custom:${esp-tools}

        export TOOLS_DIR=".embuild/espressif/tools"
        mkdir -p $TOOLS_DIR

        CMAKE_DIR=$TOOLS_DIR/cmake/3.30.2/
        XTENSA_DIR=$TOOLS_DIR/xtensa-esp-elf/esp-14.2.0_20241119/xtensa-esp-elf
        NINJA_DIR=$TOOLS_DIR/ninja/1.12.1
        ESP32ULP_DIR=$TOOLS_DIR/esp32ulp-elf/2.38_20240113/esp32ulp-elf

        mkdir -p $CMAKE_DIR
        mkdir -p $XTENSA_DIR
        mkdir -p $NINJA_DIR
        mkdir -p $ESP32ULP_DIR

        ln -s "${cmake}/bin" $CMAKE_DIR/
        ln -s "${cmake}/share" $CMAKE_DIR/

        ln -s "${idf-rust}/.rustup/toolchains/esp/xtensa-esp-elf/esp-14.2.0_20240906/xtensa-esp-elf/bin" $XTENSA_DIR/

        ln -s "${ninja}/bin/ninja" $NINJA_DIR/.

        ln -s "${esp32ulp}/bin" $ESP32ULP_DIR/
        ln -s "${esp-idf}" ./esp-idf
      '';
    };
  };
}
