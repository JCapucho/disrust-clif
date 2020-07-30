#!/bin/sh

set -e

jit_naked() {
    echo "$@" | CG_CLIF_JIT=1 CHANNEL="release" rustc +$(cat $cg_clif_dir/rust-toolchain) -Cpanic=abort -Zcodegen-backend=$cg_clif_dir/target/release/librustc_codegen_cranelift.so --sysroot $cg_clif_dir/build_sysroot/sysroot - -Cprefer-dynamic
}

"$@"
