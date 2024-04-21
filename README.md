# solstrale-desktop-rust

A desktop UI for the Solstråle path tracer

## How to build (Fedora 39)

1. Install Rust: https://rustup.rs/
2. (_Optional, for the denoise oidn-postprocessor feature to work_) Install Threading Building Blocks library and Intel®
   Open Image Denoise by `sudo dnf install tbb oidn`. If running another distribution, you might have to build oidn
   yourself. Not too hard they have good docs.
3. Clone this repo and go to root folder
4. (_Also optional, for the denoise post-processor_) Edit cargo.toml change the line `solstrale = "x.y.z"`
   to `solstrale = { version = "x.y.z", features = ["oidn-postprocessor"] }`
5. Run `./build.sh` or `./build_with_oidn.sh` and take a cup of coffee
6. Start the program by `target/release/solstrale-desktop`

![image](https://github.com/DanielPettersson/solstrale-desktop-rust/assets/3603911/432b6661-716a-46ef-86ab-3789c4fb52da)
