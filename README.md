# A multiprocess, single core operating system written in Rust.

## Environment Set-up
We need the nightly version of rust compiler. The compiler that I have tested on is the nightly version 1.34.0. As the newer versions may not be backward compatible, please install this version to build this project. The command to add this is: 
```sh
rustup override add nightly-2019-02-10
rustup component add rust-src
cargo install cargo-xbuild
cargo install bootimage --version "0.5.8"
```

## To run the tests file
```sh
bootimage test
```

This creates the testable binaries which test each aspect of the code. It is explained in detail in the video.

## Skipping the environment set-up and binaries generation
Because this is a tedious job to generate the binaries. I have already uploaded the binaries in my S3 folder. You can choose to download the binaries from there and run it in qemu directly.

The URL for that is:
http://os.utkarsh.ch.s3.amazonaws.com/index.html

## Running the binaries in qemu
If you generate the binaries in-house, the binaries goes in the location target/x86\_64-blog\_os/debug/bootimage-test-\*.bin. If you download it they will go in your custom location and modify the qemu script accordingly

```sh
qemu-system-x86_64 -drive format=raw,file=target/x86_64-blog_os/debug/bootimage-test-scheduler.bin -m 32M -serial mon:stdio -device isa-debug-exit,iobase=0xf4,iosize=0x04
```
An examle script execution. Please note the qemu will exit after printing ok because of the last -device parameter.

To run all the tests in your qemu, run this command 
```sh
for x in target/x86_64-blog_os/debug/bootimage-test-*.bin; do echo "Test $x"; qemu-system-x86_64 -drive format=raw,file=$x -m 32M -serial mon:stdio -device isa-debug-exit,iobase=0xf4,iosize=0x04 ; done
```

