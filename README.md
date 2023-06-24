### os
本项目主要参考

[onix](https://github.com/StevenBaby/onix)
[rCore](https://github.com/rcore-os/rCore)
[Writing an OS in Rust ](https://github.com/phil-opp/blog_os)

万分感谢[StevenBaby](https://github.com/StevenBaby)

### todo

- [x] vga驱动
- [x] csi颜色序列
- [x] GDT
- [x] IDT
- [x] 内中断
- [x] 外中断
- [x] 时钟
- [x] 任务切换
- [ ] 系统调用
- [ ] 锁


### 内存布局

+ 栈 0-0x10000
+ kernel 0x10000-4G


### run
```
make bochs
make qemu
```

