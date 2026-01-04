# FakeNameDemo


一个由Rust实现的Linux进程伪装工具。



### 编译

```bash
# 克隆仓库
git clone https://github.com/b1ackow1/FakeProcessNameDemo.git
cd FakeProcessNameDemo

# 编译
cargo build --release

# 运行
./target/release/FakeProcessNameDemo
```


### 验证效果

```bash
# 查看进程列表（会显示伪装后的名称）
ps aux | grep <PID>

# 查看 cmdline（argv[0]）
cat /proc/<PID>/cmdline | tr '\0' ' '

# 查看 comm
cat /proc/<PID>/comm

# 使用 ss 或 netstat 查看网络连接（如果程序绑定了端口）
ss -tulpn | grep <PID>

```



本项目采用 [MIT License](https://opensource.org/licenses/MIT) 开源。

如果这个项目对你有帮助，欢迎给个Star！

---

