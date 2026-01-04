//! # Pure Rust 进程伪装实现
//!
//! 本项目演示如何使用Rust实现Linux进程伪装，用于AWD CTF教学环境。
//!
//! ## 功能
//! - 修改进程的 argv[0]（通过栈扫描和内存操作）
//! - 修改进程的 comm 名称（通过 prctl 系统调用）
//!
//! ## 作者
//! b1ackow1 <b1ackow1@pm.me>
//!
//! ## 许可证
//! MIT License

use std::ffi::CString;

fn main() {
    unsafe {
        modify_argv0_name("fakenamefakenamefakenamefakenamefakenamefakenamefakenamefakename").unwrap();
        mofidy_comm_name("fakenamefakenamefakenamefakenamefakenamefakenamefakenamefakename").unwrap();
    }
    println!("  cat /proc/{}/cmdline | tr '\\0' ' '", std::process::id());
    println!("  ps aux | grep {}", std::process::id());
    std::thread::sleep(std::time::Duration::from_secs(60));
} 


//调用prctl函数修改comm
unsafe fn mofidy_comm_name(process_name:&str)->Result<(), String>{
     //截断长文件名到15个字符
     let truncated_name= if process_name.len()>15{
        &process_name[..15]
     }else{
        process_name
     };
     //转换为C字符串
     let c_process_name=CString::new(truncated_name)
     .map_err(|e|format!("转换错误:{}",e)).unwrap();

    //调用prctl
    unsafe{
        if libc::prctl(libc::PR_SET_NAME,c_process_name.as_ptr() as usize,0,0,0)<0{
            return Err("prctl函数调用错误".to_string());
        }
    }
    Ok(())
}

/// 获取进程栈的地址范围
///
/// 通过读取 /proc/self/maps 查找 [stack] 区域
/// 返回 (stack_start, stack_end)
fn get_stack_range()->Result<(usize,usize),String>{
    let maps=std::fs::read_to_string("/proc/self/maps").map_err(|e|format!("不能读取maps:{}",e))?;
    //查找stack行
    for line in maps.lines(){
        if line.contains("[stack]"){
            let parts:Vec<&str>=line.split_whitespace().collect();
            if let Some(range)=parts.first(){
                let addrs:Vec<&str>=range.split("-").collect();
                if addrs.len()==2{
                    let stack_start=usize::from_str_radix(addrs[0], 16).map_err(|e|format!("解析起始地址失败:{}",e))?;
                    let stack_end=usize::from_str_radix(addrs[1], 16).map_err(|e|format!("解析结束地址失败:{}",e))?;
                    return Ok((stack_start,stack_end));
                }
            }
         }
    }
    Err("未找到[stack]区域".to_string())
}


unsafe fn find_argv0_address_in_stack(_stack_start:usize,stack_end:usize)->Result<usize,String>{  
    let argv0=std::env::args().next()
    .ok_or("没有argv[0]".to_string())?;
    let argv0_bytes=argv0.as_bytes();
    //当前ubuntu系统内核分配的ARG_MAX是2M，通常最大也是2M.
    //命令:getconf ARG_MAX
    //通过饱和减法方法减去1M从栈顶开始找。搜索最后1MB的栈空间
    let search_start = stack_end.saturating_sub(1024 * 1024);
    let search_end = stack_end - argv0_bytes.len();
    println!(
        "搜索范围: 0x{:x} - 0x{:x} ({} 字节)",
        search_start,
        search_end,
        search_end - search_start
    );
    //逐字节比较
    for addr in (search_start..search_end).step_by(1){
        let ptr=addr as *const u8;
        let mut matches = true;
        for i in 0..argv0_bytes.len(){
            unsafe {
                if *ptr.add(i)!=argv0_bytes[i]{
                    matches=false;
                    break;
                }
            }
        }
        // 如果所有字节都匹配，且后面是 \0
        unsafe {
            if matches && *ptr.add(argv0_bytes.len())==0{
                return Ok(addr);
            }
        }
    }
    Err("不能在栈中找到argv[0]的地址".to_string())
}

unsafe fn calculate_argv_space(argv_start:usize)->Result<usize,String>{
    //解析cmdline和environ的长度，获取可用长度
    let cmdline = std::fs::read("/proc/self/cmdline")
        .map_err(|e| format!("Failed to read cmdline: {}", e))?;
    let environ_data = std::fs::read("/proc/self/environ")
        .map_err(|e| format!("Failed to read environ: {}", e))?;

    let argv_end = argv_start + cmdline.len();

    let environ_end = argv_end + environ_data.len();

    Ok(environ_end)
}

/// 修改 argv[0] 进程名
///
unsafe fn modify_argv0_name(process_name:&str)->Result<(),String>{
     let (stack_start,stack_end)=get_stack_range()?;
     let argv0_start = unsafe {
        find_argv0_address_in_stack(stack_start, stack_end)?
    };
     let environ_end = unsafe {
        calculate_argv_space(argv0_start)?
    };
    let total_available = environ_end - argv0_start;
    let process_name_bytes = process_name.as_bytes();

    // 检查名称长度是否超过可用空间
    if process_name_bytes.len() + 1 > total_available {
        return Err(format!(
            "进程名太长: {} bytes,可用空间: {} bytes",
            process_name_bytes.len() + 1,
            total_available
        ));
    }

    println!("修改前 argv[0]: {}", std::env::args().next().unwrap());
    println!("将要修改为: {}", process_name);
    println!("可用空间: {} bytes", total_available);

    unsafe {
        std::ptr::write_bytes(argv0_start as *mut u8, 0, total_available);

        std::ptr::copy_nonoverlapping(
            process_name_bytes.as_ptr(),
            argv0_start as *mut u8,
            process_name_bytes.len()
        );
    }

    println!("修改成功！");

    Ok(())
}

