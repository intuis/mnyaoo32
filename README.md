# SPI Display Example

This is a simple example of how to connect ESP32 with the display connected through the SPI interface.

Feel free to play around with the code and modify the program to your liking!

## Getting Started

Make sure you have `Docker`, `Visual Studio Code` and `Google Chrome` installed.

Clone this repository:

```sh
git clone https://github.com/rust-community-pl/esp32-playground.git
```

Navigate to this folder:

```sh
cd esp32-playground/spi_display_example
```

Open the folder in Visual Studio Code:

```sh
code .
```

In the bottom-right corner press `Reopen in Container` button to open the workspace in DevContainer.

After all necessary dependencies finish installing, you are good to go!

Head to the `src/main.rs` file to see the code. You can also try to change the `src/rustmeet.bmp` image to your own.

## Building

When you are ready to build the project and flash it onto the ESP32, open a new terminal in `Visual Studio Code` inside a container and type:

```sh
cargo run
```

The project will build and when successful, will open a `web-flash` utility, so there's no need to redirect USB ports into the container.

If the url opens in a different browser, open `Google Chrome` and navigate to:

```
http://localhost:8000
```

> [!WARNING]
> Below steps will flash the current program inside the ESP32 device, proceed with caution

Press the `Connect` button, choose a proper device, and then press `Install ESP Application`. Then, you have to check the `Erase device` box, and proceed by clicking `Next`. Lastly, confirm the operatation by clicking `Install`.
