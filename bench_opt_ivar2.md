# ruruby benchmark results

## environment

Ruby version: 3.0.0  
CPU: Intel(R) Core(TM) i7-8700K CPU @ 3.70GHz  
OS: Ubuntu 20.04.1 LTS

## execution time

|     benchmark      |      ruby      |     ruruby     |  rate  |
| :----------------: | :------------: | :------------: | :----: |
|  accessor_get.rb   | 0.59 ± 0.01 s  | 1.24 ± 0.02 s  | x 2.11 |
|  accessor_set.rb   | 0.43 ± 0.01 s  | 1.36 ± 0.01 s  | x 3.17 |
|    ivar_get.rb     | 1.74 ± 0.01 s  | 1.94 ± 0.01 s  | x 1.12 |
|    ivar_set.rb     | 1.16 ± 0.02 s  | 2.09 ± 0.02 s  | x 1.80 |
|   loop_times.rb    | 0.76 ± 0.03 s  | 0.43 ± 0.01 s  | x 0.56 |
|    loop_for.rb     | 0.86 ± 0.05 s  | 0.49 ± 0.01 s  | x 0.57 |
| loop_whileloop.rb  | 0.41 ± 0.00 s  | 0.63 ± 0.01 s  | x 1.53 |
| so_concatenate.rb  | 0.72 ± 0.02 s  | 0.65 ± 0.01 s  | x 0.91 |
| string_scan_str.rb | 1.14 ± 0.02 s  | 1.08 ± 0.01 s  | x 0.95 |
| string_scan_re.rb  | 1.60 ± 0.02 s  | 1.08 ± 0.02 s  | x 0.68 |
| fiber_allocate.rb  | 1.48 ± 0.06 s  | 0.92 ± 0.01 s  | x 0.62 |
|  fiber_switch.rb   | 0.79 ± 0.00 s  | 1.08 ± 0.03 s  | x 1.38 |
|  so_mandelbrot.rb  | 1.82 ± 0.01 s  | 2.62 ± 0.14 s  | x 1.44 |
| app_mandelbrot.rb  | 1.39 ± 0.04 s  | 1.16 ± 0.01 s  | x 0.84 |
|    app_fibo.rb     | 0.56 ± 0.01 s  | 1.36 ± 0.01 s  | x 2.45 |
|   app_aobench.rb   | 10.38 ± 0.20 s | 20.32 ± 0.52 s | x 1.96 |
|    so_nbody.rb     | 1.08 ± 0.05 s  | 2.62 ± 0.20 s  | x 2.42 |
|     collatz.rb     | 6.50 ± 0.25 s  | 8.37 ± 0.22 s  | x 1.29 |

## optcarrot benchmark

|    benchmark    |       ruby        |      ruruby      |  rate  |
| :-------------: | :---------------: | :--------------: | :----: |
|    optcarrot    | 40.08 ± 1.87 fps  | 9.63 ± 1.88 fps  | x 4.16 |
| optcarrot --opt | 126.10 ± 3.42 fps | 56.83 ± 4.56 fps | x 2.22 |

## memory consumption

|     benchmark      | ruby  | ruruby |  rate   |
| :----------------: | :---: | :----: | :-----: |
|  accessor_get.rb   | 22.0M |  4.9M  | x 0.22  |
|  accessor_set.rb   | 22.1M |  4.9M  | x 0.22  |
|    ivar_get.rb     | 22.1M |  5.0M  | x 0.22  |
|    ivar_set.rb     | 22.0M |  4.9M  | x 0.22  |
|   loop_times.rb    | 22.1M |  4.9M  | x 0.22  |
|    loop_for.rb     | 22.0M |  4.9M  | x 0.22  |
| loop_whileloop.rb  | 22.1M |  4.9M  | x 0.22  |
| so_concatenate.rb  | 72.6M | 63.6M  | x 0.88  |
| string_scan_str.rb | 27.1M |  6.6M  | x 0.24  |
| string_scan_re.rb  | 26.7M |  6.6M  | x 0.25  |
| fiber_allocate.rb  | 49.2M | 224.8M | x 4.56  |
|  fiber_switch.rb   | 22.0M |  5.0M  | x 0.23  |
|  so_mandelbrot.rb  | 22.2M |  5.8M  | x 0.26  |
| app_mandelbrot.rb  | 22.1M |  5.7M  | x 0.26  |
|    app_fibo.rb     | 22.1M |  4.9M  | x 0.22  |
|   app_aobench.rb   | 22.6M |  7.1M  | x 0.32  |
|    so_nbody.rb     | 22.1M |  5.3M  | x 0.24  |
|     collatz.rb     | 22.0M |  5.0M  | x 0.22  |
|     optcarrot      | 78.7M | 64.2M  | x 0.82  |
|  optcarrot --opt   | 80.0M | 811.5M | x 10.14 |
