FROM rustlang/rust:nightly as cranelift

WORKDIR /usr/src/disrust

COPY . .

RUN git clone https://github.com/bjorn3/rustc_codegen_cranelift.git
RUN git config --global user.email "you@example.com"
RUN git config --global user.name "Your Name"

WORKDIR /usr/src/disrust/rustc_codegen_cranelift

RUN git checkout ce047701241c8f1eb485d2395a0ec615361ca8ef
RUN git am --signoff < ../0001-Remove-jit-message.patch

RUN ./prepare.sh
ENV CHANNEL='release'
RUN CARGO_INCREMENTAL=1 cargo rustc --release -- -Zrun_dsymutil=no

RUN chmod +x ./config.sh
RUN bash ./config.sh
RUN ./build_sysroot/build_sysroot.sh --release

FROM cranelift

WORKDIR /usr/src/disrust

ENV CFG_RELEASE=1.47.0-nightly
ENV CFG_RELEASE_CHANNEL=nightly
ENV cg_clif_dir=/usr/src/disrust/rustc_codegen_cranelift

RUN cargo install --path .

RUN useradd ferris

USER ferris

CMD ["disrust"]