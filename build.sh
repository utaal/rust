#!/bin/bash

set -e

unamestr=`uname`
if [[ "$unamestr" == 'Linux' ]]; then
   dylib_ext='so'
elif [[ "$unamestr" == 'Darwin' ]]; then
   dylib_ext='dylib'
else
   echo "Unsupported os"
   exit 1
fi

build_example_bin() {
    $RUSTC $2 --crate-name $1 --crate-type bin

    pushd target/out
    gcc $1 libmini_core.rlib -o $1_bin
    sh -c ./$1_bin || true
    popd
}

if [[ "$1" == "--release" ]]; then
    channel='release'
    cargo build --release
else
    channel='debug'
    cargo build
fi

export RUSTFLAGS='-Zalways-encode-mir -Cpanic=abort -Zcodegen-backend='$(pwd)'/target/'$channel'/librustc_codegen_cranelift.'$dylib_ext
export XARGO_RUST_SRC=$(pwd)'/target/libcore/src'
RUSTC="rustc $RUSTFLAGS -L crate=target/out --out-dir target/out"

rm -r target/out || true
mkdir -p target/out/clif

echo "[BUILD] mini_core"
$RUSTC example/mini_core.rs --crate-name mini_core --crate-type lib

echo "[BUILD] example"
$RUSTC example/example.rs --crate-type lib

echo "[JIT] mini_core_hello_world"
SHOULD_RUN=1 $RUSTC --crate-type bin example/mini_core_hello_world.rs --cfg jit

echo "[AOT] mini_core_hello_world"
build_example_bin mini_core_hello_world example/mini_core_hello_world.rs

pushd xargo
rm -r ~/.xargo/HOST || true
rm -r target || true
time xargo build
popd

$RUSTC --sysroot ~/.xargo/HOST example/alloc_example.rs --crate-type bin
# TODO linux linker doesn't accept duplicate definitions
#gcc -Wl,--gc-sections target/out/alloc_example ~/.xargo/HOST/lib/rustlib/*/lib/libcore-*.rlib ~/.xargo/HOST/lib/rustlib/*/lib/liballoc-*.rlib ~/.xargo/HOST/lib/rustlib/*/lib/liballoc_system-*.rlib -o target/out/alloc_example_exe
#hyperfine ./target/out/alloc_example_exe

$RUSTC --sysroot ~/.xargo/HOST example/mod_bench.rs --crate-type bin
gcc -Wl,--gc-sections target/out/mod_bench -lc -o target/out/mod_bench_exe

$RUSTC --sysroot ~/.xargo/HOST example/mod_bench.rs --crate-type bin -Zmir-opt-level=3 --crate-name mod_bench_inline
gcc -Wl,--gc-sections target/out/mod_bench_inline -lc -o target/out/mod_bench_inline_exe

rustc example/mod_bench.rs --crate-type bin -Copt-level=0 -o target/out/mod_bench_llvm_0 -Cpanic=abort
rustc example/mod_bench.rs --crate-type bin -Copt-level=1 -o target/out/mod_bench_llvm_1 -Cpanic=abort
rustc example/mod_bench.rs --crate-type bin -Copt-level=2 -o target/out/mod_bench_llvm_2 -Cpanic=abort
rustc example/mod_bench.rs --crate-type bin -Copt-level=3 -o target/out/mod_bench_llvm_3 -Cpanic=abort
echo
echo "[Bench] mod_bench"
hyperfine ./target/out/mod_bench{,_inline}_exe ./target/out/mod_bench_llvm_*

cat target/out/log.txt | sort | uniq -c
