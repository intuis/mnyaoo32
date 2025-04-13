{ stdenv, lib, fetchurl, makeWrapper, buildFHSUserEnv }:

let
  fhsEnv = buildFHSUserEnv {
    name = "esp32-ulp-elf-env";
    targetPkgs = pkgs: with pkgs; [ zlib ];
    runScript = "";
  };
in

stdenv.mkDerivation rec {
  pname = "esp32-ulp-elf";
  version = "2.38_20240113";

  src = fetchurl {
#    url = "https://github.com/espressif/crosstool-NG/releases/download/esp-${version}/riscv32-esp-elf-gcc8_4_0-esp-${version}-linux-amd64.tar.gz";
#    hash = "sha256-F5y61Xl5CtNeD0FKGNkAF8DxWMOXAiQRqOmGfbIXTxU=";
    url = "https://github.com/espressif/binutils-gdb/releases/download/esp32ulp-elf-${version}/esp32ulp-elf-${version}-linux-amd64.tar.gz";
    hash = "sha256-0TqAg2W3hGX6ZZFjbfu7lgTZ0Vo5fD2c0iYm1UgorCw=";
  };

  buildInputs = [ makeWrapper ];

  phases = [ "unpackPhase" "installPhase" ];

  installPhase = ''
    cp -r . $out
    for FILE in $(ls $out/bin); do
      FILE_PATH="$out/bin/$FILE"
      if [[ -x $FILE_PATH ]]; then
        mv $FILE_PATH $FILE_PATH-unwrapped
        makeWrapper ${fhsEnv}/bin/esp32-ulp-elf-env $FILE_PATH --add-flags "$FILE_PATH-unwrapped"
      fi
    done
  '';

  meta = with lib; {
    description = "ESP32 toolchain";
    homepage = https://docs.espressif.com/projects/esp-idf/en/stable/get-started/linux-setup.html;
    license = licenses.gpl3;
  };
}
