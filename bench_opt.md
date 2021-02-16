# ruruby benchmark results

## environment

Ruby version: 3.0.0  
CPU: Apple M1  
OS: macOS 11.1

## execution time

|     benchmark      |     ruby      |     ruruby     |  rate  |
| :----------------: | :-----------: | :------------: | :----: |
|   loop_times.rb    | 0.73 ± 0.02 s | 0.35 ± 0.17 s  | x 0.48 |
|    loop_for.rb     | 0.78 ± 0.01 s | 0.33 ± 0.00 s  | x 0.42 |
| loop_whileloop.rb  | 0.52 ± 0.01 s | 0.36 ± 0.00 s  | x 0.69 |
| so_concatenate.rb  | 1.97 ± 0.01 s | 1.96 ± 0.01 s  | x 0.99 |
| string_scan_str.rb | 0.63 ± 0.00 s | 0.76 ± 0.00 s  | x 1.21 |
| string_scan_re.rb  | 0.95 ± 0.01 s | 0.76 ± 0.00 s  | x 0.80 |
| fiber_allocate.rb  | 1.16 ± 0.01 s | 1.04 ± 0.01 s  | x 0.89 |
|  fiber_switch.rb   | 1.26 ± 0.00 s | 1.65 ± 0.00 s  | x 1.31 |
|  so_mandelbrot.rb  | 1.18 ± 0.04 s | 1.26 ± 0.01 s  | x 1.06 |
| app_mandelbrot.rb  | 0.83 ± 0.01 s | 0.81 ± 0.01 s  | x 0.98 |
|    app_fibo.rb     | 0.54 ± 0.00 s | 0.91 ± 0.00 s  | x 1.69 |
|   app_aobench.rb   | 5.82 ± 0.01 s | 12.40 ± 0.07 s | x 2.13 |
|    so_nbody.rb     | 0.77 ± 0.01 s | 1.49 ± 0.15 s  | x 1.93 |
|     collatz.rb     | 5.84 ± 0.04 s | 5.95 ± 0.02 s  | x 1.02 |

## optcarrot benchmark

|    benchmark    |       ruby        |      ruruby      |  rate  |
| :-------------: | :---------------: | :--------------: | :----: |
|    optcarrot    | 56.26 ± 0.62 fps  | 20.86 ± 0.05 fps | x 2.70 |
| optcarrot --opt | 129.93 ± 0.79 fps | 82.76 ± 5.16 fps | x 1.57 |

## memory consumption

|     benchmark      |  ruby  | ruruby |  rate  |
| :----------------: | :----: | :----: | :----: |
|   loop_times.rb    | 26.6M  |  3.1M  | x 0.12 |
|    loop_for.rb     | 26.5M  |  3.1M  | x 0.12 |
| loop_whileloop.rb  | 26.5M  |  3.1M  | x 0.12 |
| so_concatenate.rb  | 134.9M | 200.6M | x 1.49 |
| string_scan_str.rb | 28.3M  |  5.3M  | x 0.19 |
| string_scan_re.rb  | 28.4M  |  5.5M  | x 0.19 |
| fiber_allocate.rb  | 211.5M | 328.8M | x 1.55 |
|  fiber_switch.rb   | 26.5M  |  3.2M  | x 0.12 |
|  so_mandelbrot.rb  | 26.9M  |  4.0M  | x 0.15 |
| app_mandelbrot.rb  | 27.1M  |  3.9M  | x 0.15 |
|    app_fibo.rb     | 26.6M  |  3.2M  | x 0.12 |
|   app_aobench.rb   | 27.0M  |  5.7M  | x 0.21 |
|    so_nbody.rb     | 26.5M  |  3.5M  | x 0.13 |
|     collatz.rb     | 26.5M  |  3.2M  | x 0.12 |
|     optcarrot      | 100.5M | 65.1M  | x 0.65 |
|  optcarrot --opt   | 111.3M | 998.8M | x 8.97 |
