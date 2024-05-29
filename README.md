validate that the logic is still correct

```
cargo test --all-features --target x86_64-unknown-linux-gnu
```

This change saves a lot of code!

```
> cargo bloat --release --features "left-std"

File  .text   Size        Crate Name
2.1%  51.1%   830B          std compiler_builtins::mem::memmove
0.8%  20.2%   328B          std compiler_builtins::mem::memcpy
0.7%  17.7%   288B slice_rotate slice_rotate::__cortex_m_rt_main
0.2%   3.8%    62B  cortex_m_rt Reset
0.1%   1.5%    24B    [Unknown] HardFaultTrampoline
0.1%   1.5%    24B          std core::ptr::swap_nonoverlapping
0.0%   0.5%     8B          std core::panicking::panic
0.0%   0.5%     8B    [Unknown] main
0.0%   0.2%     4B    [Unknown] __aeabi_memcpy
0.0%   0.2%     4B          std compiler_builtins::arm::__aeabi_memmove
0.0%   0.2%     4B          std compiler_builtins::arm::__aeabi_memcpy
0.0%   0.2%     4B    [Unknown] __aeabi_memmove
0.0%   0.1%     2B  cortex_m_rt HardFault_
0.0%   0.1%     2B  cortex_m_rt DefaultPreInit
0.0%   0.1%     2B  cortex_m_rt DefaultHandler_
0.0%   0.1%     2B          std core::panicking::panic_fmt
0.0%   0.0%     0B              And 0 smaller methods. Use -n N to show more.
4.1% 100.0% 1.6KiB              .text section size, the file size is 38.8KiB

> cargo bloat --release --features "left-size"

File  .text Size        Crate Name
0.8%  42.0% 116B slice_rotate slice_rotate::__cortex_m_rt_main
0.4%  22.5%  62B  cortex_m_rt Reset
0.2%   8.7%  24B    [Unknown] HardFaultTrampoline
0.2%   8.7%  24B          std core::ptr::swap_nonoverlapping
0.1%   2.9%   8B          std core::panicking::panic
0.1%   2.9%   8B    [Unknown] main
0.0%   0.7%   2B  cortex_m_rt HardFault_
0.0%   0.7%   2B  cortex_m_rt DefaultPreInit
0.0%   0.7%   2B  cortex_m_rt DefaultHandler_
0.0%   0.7%   2B          std core::panicking::panic_fmt
0.0%   0.0%   0B              And 0 smaller methods. Use -n N to show more.
1.9% 100.0% 276B              .text section size, the file size is 14.5KiB
```
