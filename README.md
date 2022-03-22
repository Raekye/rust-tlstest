# tlstest
Inspecting the generated assembly for thread-local storage access on amd64 Linux.

This Cargo project consists of a library crate with a binary executable.
The library has a private, thread-local variable modifiable via public functions (`LIB_TL` in `src/lib.rs`).
The binary also has a thread-local variable (`BIN_TL` in `src/bin/tlstest.rs`).

First, build in release mode: `cargo build --release`.

Dump the assembly for accessing the thread-local variable defined in the binary: `objdump --disassemble=get_bin_tl target/release/tlstest`.
It should look something like this:

```
00000000000075d0 <get_bin_tl>:
    75d0:       64 48 8b 04 25 b8 ff    mov    %fs:0xffffffffffffffb8,%rax
    75d7:       ff ff
    75d9:       c3                      ret
```

Note that `0xffffffffffffffb8` is `-0x48`.
As far as I understand, `mov %fs:-0x48,%rax` loads the word at offset `-0x48` from the value in the `%fs` register into `%rax`.

Dump the assembly for accessing the thread-local variable defined in the library: `objdump --disassemble=get_lib_tl target/release/tlstest`.
It should look something like this:

```
0000000000007710 <get_lib_tl>:
    7710:       50                      push   %rax
    7711:       66 66 66 64 48 8b 04    data16 data16 data16 mov %fs:0x0,%rax
    7718:       25 00 00 00 00
    771d:       48 8b 80 c0 ff ff ff    mov    -0x40(%rax),%rax
    7724:       59                      pop    %rcx
    7725:       c3                      ret
```

As far as I understand, the `data16` are just padding.

The curious thing is that (as far as I understand) `%fs:0x0`,
the word at offset 0 from the value in `%fs`,
just contains the value of `%fs` itself (see [section 4.4.6 of "ELF Handling for Thread-Local Storage"][1]);
i.e. after `mov %fs:0x0,%rax`, the `%rax` register contains the same value as the `%fs` segment register.
I've heard that the reason for this is because one cannot easily extract the value of the `%fs` register directly.
If that is true, then the instructions `mov %fs:0x0,%rax`, `mov -0x40(%rax),%rax`,
compute the same value into `%rax` as just `mov %fs:-0x40,%rax`.
In other words, I don't understand why `get_lib_tl` doesn't look like `get_bin_tl` like this:

```
mov %fs:-0x40,%rax
ret
```

The page on [`rustc`'s TLS model][2] says:

- `initial-exec` - model usable if the TLS data is defined in the executable or in a shared library loaded at program startup. The TLS data must not be in a library loaded after startup (via `dlopen`).
- `local-exec` - model usable only if the TLS data is defined directly in the executable, but not in a shared library, and is accessed only from that executable.

As far as I understand, `lib.rs` is **not** being compiled as a shared library, at least when compiled into the `tlstest.rs` binary;
the `local-exec` model is used (as explained in ["ELF Handling for Thread-Local Storage"][1]).
In any case, both instruction sequences (for both `get_bin_tl` and `get_lib_tl`)
match examples in the "local exec" section of "ELF Handling for Thread-Local Storage",
with `get_bin_tl` containing the "short" version.
The assembly for `get_lib_tl` does not look like  the assembly for "initial exec" (section 4.3.6),
so I do not believe the "initial exec" model is being used.

Here is another explanation of thread-local variables: ["A Deep dive into (implicit) Thread Local Storage"][3],
which also goes over the 4 TLS models (the other two being almost certainly irrelevant to this code).

So my question is: why does Rust/LLVM generate the (slightly) longer sequence for thread-local access inside a non-shared(?), statically-linked(?) library,
as opposed to when the thread-local variable is defined inside the executable itself?

[1]: https://www.akkadia.org/drepper/tls.pdf
[2]: https://doc.rust-lang.org/beta/unstable-book/compiler-flags/tls-model.html
[3]: https://chao-tic.github.io/blog/2018/12/25/tls
