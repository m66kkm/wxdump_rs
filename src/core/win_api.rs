// src/core/win_api.rs

use anyhow::{Result, anyhow};
use windows_sys::Win32::{
    Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE},
    System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
    },
};

#[derive(Debug)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
}

/// Lists all running processes.
pub fn list_processes() -> Result<Vec<ProcessInfo>> {
    let mut processes = Vec::new();
    let snapshot_handle: HANDLE = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };

    if snapshot_handle == INVALID_HANDLE_VALUE {
        return Err(anyhow!("Failed to create toolhelp snapshot. Error: {}", std::io::Error::last_os_error()));
    }

    let mut process_entry: PROCESSENTRY32W = unsafe { std::mem::zeroed() };
    process_entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

    if unsafe { Process32FirstW(snapshot_handle, &mut process_entry) } == 0 {
        unsafe { CloseHandle(snapshot_handle) };
        return Err(anyhow!("Failed to get first process. Error: {}", std::io::Error::last_os_error()));
    }

    loop {
        let process_name = String::from_utf16_lossy(&process_entry.szExeFile)
            .trim_end_matches('\0') // Remove null terminators
            .to_string();
        
        processes.push(ProcessInfo {
            pid: process_entry.th32ProcessID,
            name: process_name,
        });

        if unsafe { Process32NextW(snapshot_handle, &mut process_entry) } == 0 {
            break;
        }
    }

    unsafe { CloseHandle(snapshot_handle) };
    Ok(processes)
}

/// Gets the executable path for a given process ID.
pub fn get_process_exe_path(pid: u32) -> Result<String> {
    const MAX_PATH_LEN: usize = 1024; // Increased buffer size
    let mut exe_path_bytes: Vec<u16> = vec![0; MAX_PATH_LEN];

    let process_handle: HANDLE = unsafe {
        windows_sys::Win32::System::Threading::OpenProcess(
            windows_sys::Win32::System::Threading::PROCESS_QUERY_INFORMATION | windows_sys::Win32::System::Threading::PROCESS_VM_READ,
            0, // FALSE (bInheritHandle)
            pid,
        )
    };

    if process_handle == std::ptr::null_mut() || process_handle == INVALID_HANDLE_VALUE {
        return Err(anyhow!("Failed to open process {}. Error: {}", pid, std::io::Error::last_os_error()));
    }

    let buffer_size = MAX_PATH_LEN as u32;
    // K32GetModuleFileNameExW returns the length of the string copied to the buffer
    // (excluding the null terminator) upon success, or 0 on failure.
    let actual_len_copied = unsafe {
        windows_sys::Win32::System::ProcessStatus::K32GetModuleFileNameExW(
            process_handle,
            std::ptr::null_mut(), // hModule, NULL for the main executable. HMODULE is *mut c_void.
            exe_path_bytes.as_mut_ptr(),
            buffer_size, // Pass the buffer size
        )
    };

    unsafe { CloseHandle(process_handle) };

    if actual_len_copied == 0 { // If the function fails, it returns 0
        return Err(anyhow!("Failed to get process exe path for PID {}. Error: {}", pid, std::io::Error::last_os_error()));
    }

    // Convert Vec<u16> to String, using the actual length returned by K32GetModuleFileNameExW
    let exe_path = String::from_utf16_lossy(&exe_path_bytes[..actual_len_copied as usize]);
    Ok(exe_path.trim_end_matches('\0').to_string()) // trim_end_matches is good practice, though K32...ExW's length doesn't include it.
}

#[allow(non_snake_case)] // Allow non_snake_case for Windows API struct
#[repr(C)]
struct VS_FIXEDFILEINFO {
    dwSignature: u32,
    dwStrucVersion: u32,
    dwFileVersionMS: u32,
    dwFileVersionLS: u32,
    dwProductVersionMS: u32,
    dwProductVersionLS: u32,
    dwFileFlagsMask: u32,
    dwFileFlags: u32,
    dwFileOS: u32,
    dwFileType: u32,
    dwFileSubtype: u32,
    dwFileDateMS: u32,
    dwFileDateLS: u32,
}

/// Gets the file version information for a given executable path.
pub fn get_file_version_info(exe_path: &str) -> Result<String> {
    let mut wide_path: Vec<u16> = exe_path.encode_utf16().chain(std::iter::once(0)).collect();
    let mut dummy_handle: u32 = 0; // This parameter is not used by GetFileVersionInfoSizeW and can be zero.

    let version_info_size = unsafe {
        windows_sys::Win32::Storage::FileSystem::GetFileVersionInfoSizeW(wide_path.as_mut_ptr(), &mut dummy_handle)
    };

    if version_info_size == 0 {
        return Err(anyhow!("Failed to get file version info size for [{}]. Error: {}", exe_path, std::io::Error::last_os_error()));
    }

    let mut version_info_buffer: Vec<u8> = vec![0; version_info_size as usize];

    let success = unsafe {
        windows_sys::Win32::Storage::FileSystem::GetFileVersionInfoW(
            wide_path.as_mut_ptr(),
            0, // This parameter is not used and should be zero.
            version_info_size,
            version_info_buffer.as_mut_ptr() as *mut std::ffi::c_void,
        )
    };

    if success == 0 { // Returns 0 on failure
        return Err(anyhow!("Failed to get file version info for [{}]. Error: {}", exe_path, std::io::Error::last_os_error()));
    }

    let mut fixed_file_info_ptr: *mut VS_FIXEDFILEINFO = std::ptr::null_mut();
    let mut len: u32 = 0;
    let query_success = unsafe {
        windows_sys::Win32::Storage::FileSystem::VerQueryValueW(
            version_info_buffer.as_ptr() as *const std::ffi::c_void,
            "\\".encode_utf16().chain(std::iter::once(0)).collect::<Vec<u16>>().as_ptr(),
            &mut fixed_file_info_ptr as *mut _ as *mut *mut std::ffi::c_void, // Pointer to a pointer
            &mut len,
        )
    };

    if query_success == 0 || fixed_file_info_ptr.is_null() || len == 0 {
        return Err(anyhow!("Failed to query VS_FIXEDFILEINFO from version info for [{}]. Error: {}", exe_path, std::io::Error::last_os_error()));
    }
    
    let fixed_file_info = unsafe { &*fixed_file_info_ptr };

    // dwSignature should be 0xFEEF04BD
    if fixed_file_info.dwSignature != 0xFEEF04BD {
        return Err(anyhow!("Invalid VS_FIXEDFILEINFO signature for [{}]", exe_path));
    }

    let major = (fixed_file_info.dwFileVersionMS >> 16) & 0xffff;
    let minor = fixed_file_info.dwFileVersionMS & 0xffff;
    let build = (fixed_file_info.dwFileVersionLS >> 16) & 0xffff;
    let patch = fixed_file_info.dwFileVersionLS & 0xffff;

    Ok(format!("{}.{}.{}.{}", major, minor, build, patch))
}

/// Reads a region of memory from a specified process.
pub fn read_process_memory(pid: u32, address: usize, size: usize) -> Result<Vec<u8>> {
    if size == 0 {
        return Ok(Vec::new());
    }

    let process_handle: HANDLE = unsafe {
        windows_sys::Win32::System::Threading::OpenProcess(
            windows_sys::Win32::System::Threading::PROCESS_VM_READ, // Only need VM_READ for this
            0, // FALSE (bInheritHandle)
            pid,
        )
    };

    if process_handle == std::ptr::null_mut() || process_handle == INVALID_HANDLE_VALUE {
        return Err(anyhow!("Failed to open process {} for reading memory. Error: {}", pid, std::io::Error::last_os_error()));
    }

    let mut buffer: Vec<u8> = vec![0; size];
    let mut bytes_read: usize = 0;

    let success = unsafe {
        windows_sys::Win32::System::Diagnostics::Debug::ReadProcessMemory(
            process_handle,
            address as *const std::ffi::c_void, // Base address to read from
            buffer.as_mut_ptr() as *mut std::ffi::c_void, // Buffer to store read data
            size, // Number of bytes to read
            &mut bytes_read, // Number of bytes actually read
        )
    };

    unsafe { CloseHandle(process_handle) };

    if success == 0 { // Returns 0 on failure
        return Err(anyhow!(
            "Failed to read process memory for PID {} at address 0x{:X}. Bytes to read: {}. Error: {}",
            pid, address, size, std::io::Error::last_os_error()
        ));
    }

    // It's possible that less bytes were read than requested if the region is smaller
    // than `size` or if part of it is inaccessible.
    // We should resize the buffer to the actual number of bytes read.
    buffer.truncate(bytes_read);

    Ok(buffer)
}

/// Gets the base address of a specific module loaded in a process.
pub fn get_module_base_address(pid: u32, module_name: &str) -> Result<usize> {
    let snapshot_handle: HANDLE = unsafe {
        windows_sys::Win32::System::Diagnostics::ToolHelp::CreateToolhelp32Snapshot(
            windows_sys::Win32::System::Diagnostics::ToolHelp::TH32CS_SNAPMODULE |
            windows_sys::Win32::System::Diagnostics::ToolHelp::TH32CS_SNAPMODULE32,
            pid,
        )
    };

    if snapshot_handle == INVALID_HANDLE_VALUE {
        return Err(anyhow!(
            "Failed to create module snapshot for PID {}. Error: {}",
            pid, std::io::Error::last_os_error()
        ));
    }

    let mut module_entry: windows_sys::Win32::System::Diagnostics::ToolHelp::MODULEENTRY32W = unsafe { std::mem::zeroed() };
    module_entry.dwSize = std::mem::size_of::<windows_sys::Win32::System::Diagnostics::ToolHelp::MODULEENTRY32W>() as u32;

    if unsafe { windows_sys::Win32::System::Diagnostics::ToolHelp::Module32FirstW(snapshot_handle, &mut module_entry) } == 0 {
        unsafe { CloseHandle(snapshot_handle) };
        return Err(anyhow!(
            "Failed to get first module for PID {}. Error: {}",
            pid, std::io::Error::last_os_error()
        ));
    }

    let mut found_base_address: Option<usize> = None;
    loop {
        let current_module_name = String::from_utf16_lossy(&module_entry.szModule)
            .trim_end_matches('\0')
            .to_string();
        
        if current_module_name.eq_ignore_ascii_case(module_name) {
            found_base_address = Some(module_entry.modBaseAddr as usize);
            break;
        }

        if unsafe { windows_sys::Win32::System::Diagnostics::ToolHelp::Module32NextW(snapshot_handle, &mut module_entry) } == 0 {
            break;
        }
    }

    unsafe { CloseHandle(snapshot_handle) };

    match found_base_address {
        Some(addr) => Ok(addr),
        None => Err(anyhow!("Module '{}' not found in PID {}", module_name, pid)),
    }
}

/// Determines the pointer size (4 for 32-bit, 8 for 64-bit) for a given process.
pub fn get_process_architecture(pid: u32) -> Result<usize> {
    let process_handle = unsafe {
        windows_sys::Win32::System::Threading::OpenProcess(
            windows_sys::Win32::System::Threading::PROCESS_QUERY_LIMITED_INFORMATION,
            0, // FALSE
            pid,
        )
    };
    if process_handle == std::ptr::null_mut() || process_handle == INVALID_HANDLE_VALUE {
        return Err(anyhow!("Failed to open process {} to determine architecture. Error: {}", pid, std::io::Error::last_os_error()));
    }

    let mut is_wow64: windows_sys::Win32::Foundation::BOOL = 0;
    // IsWow64Process is used to check if a 32-bit process is running on a 64-bit system.
    let success_wow64 = unsafe {
        windows_sys::Win32::System::Threading::IsWow64Process(process_handle, &mut is_wow64)
    };
    unsafe { CloseHandle(process_handle) }; // Close handle as soon as it's no longer needed

    if success_wow64 == 0 { // 0 indicates failure for IsWow64Process
        return Err(anyhow!("IsWow64Process failed for PID {}. Error: {}", pid, std::io::Error::last_os_error()));
    }

    if is_wow64 != 0 { // Non-zero (TRUE) means it's a 32-bit process on a 64-bit OS
        Ok(4) // 32-bit pointer size
    } else {
        // If not WOW64, the process architecture matches the OS architecture.
        // We need to check the OS architecture.
        let mut system_info: windows_sys::Win32::System::SystemInformation::SYSTEM_INFO = unsafe { std::mem::zeroed() };
        unsafe { windows_sys::Win32::System::SystemInformation::GetNativeSystemInfo(&mut system_info) };
        
        // Accessing union fields is unsafe
        let processor_architecture = unsafe { system_info.Anonymous.Anonymous.wProcessorArchitecture }; // This line is correct
        match processor_architecture { // The match itself doesn't need to be in an unsafe block if the value is already extracted
            windows_sys::Win32::System::SystemInformation::PROCESSOR_ARCHITECTURE_AMD64 |
            windows_sys::Win32::System::SystemInformation::PROCESSOR_ARCHITECTURE_IA64 |
            windows_sys::Win32::System::SystemInformation::PROCESSOR_ARCHITECTURE_ARM64 => Ok(8), // 64-bit OS, so process is 64-bit
            
            windows_sys::Win32::System::SystemInformation::PROCESSOR_ARCHITECTURE_INTEL |
            windows_sys::Win32::System::SystemInformation::PROCESSOR_ARCHITECTURE_ARM => Ok(4),    // 32-bit OS, so process is 32-bit
            
            arch_val => Err(anyhow!("Unknown or unsupported processor architecture: {}", arch_val)),
        }
    }
}

/// Searches for a byte pattern within a given memory region of a process.
/// Note: This is a basic implementation. For large processes or frequent searches,
/// more optimized searching algorithms and careful consideration of memory regions are needed.
pub fn search_memory_for_pattern(
    pid: u32,
    pattern: &[u8],
    start_address: usize,
    end_address: usize,
    max_occurrences: usize,
) -> Result<Vec<usize>> {
    if pattern.is_empty() {
        return Ok(Vec::new());
    }

    let process_handle = unsafe {
        windows_sys::Win32::System::Threading::OpenProcess(
            windows_sys::Win32::System::Threading::PROCESS_VM_READ | windows_sys::Win32::System::Threading::PROCESS_QUERY_INFORMATION,
            0, // FALSE
            pid,
        )
    };
    if process_handle == std::ptr::null_mut() || process_handle == INVALID_HANDLE_VALUE {
        return Err(anyhow!("Failed to open process {} for memory search. Error: {}", pid, std::io::Error::last_os_error()));
    }

    let mut found_addresses = Vec::new();
    let mut current_address = start_address;
    let mut buffer = vec![0u8; 4096 * 2]; // Read in chunks (e.g., 8KB)

    while current_address < end_address && found_addresses.len() < max_occurrences {
        let mut mem_info: windows_sys::Win32::System::Memory::MEMORY_BASIC_INFORMATION = unsafe { std::mem::zeroed() };
        let query_result = unsafe {
            windows_sys::Win32::System::Memory::VirtualQueryEx(
                process_handle,
                current_address as *const std::ffi::c_void,
                &mut mem_info,
                std::mem::size_of::<windows_sys::Win32::System::Memory::MEMORY_BASIC_INFORMATION>(),
            )
        };

        if query_result == 0 {
            // Cannot query this region, or end of address space for process
            break;
        }

        // Only read from committed memory that is readable
        if mem_info.State == windows_sys::Win32::System::Memory::MEM_COMMIT &&
           (mem_info.Protect == windows_sys::Win32::System::Memory::PAGE_READWRITE ||
            mem_info.Protect == windows_sys::Win32::System::Memory::PAGE_READONLY ||
            mem_info.Protect == windows_sys::Win32::System::Memory::PAGE_EXECUTE_READ ||
            mem_info.Protect == windows_sys::Win32::System::Memory::PAGE_EXECUTE_READWRITE) {
            
            let region_base = mem_info.BaseAddress as usize;
            let region_end = region_base + mem_info.RegionSize;
            let mut address_in_region_to_scan = current_address;

            while address_in_region_to_scan < region_end && found_addresses.len() < max_occurrences {
                let bytes_to_read = std::cmp::min(buffer.len(), region_end - address_in_region_to_scan);
                if bytes_to_read == 0 { break; }

                let mut bytes_read_count: usize = 0;
                let read_success = unsafe {
                    windows_sys::Win32::System::Diagnostics::Debug::ReadProcessMemory(
                        process_handle,
                        address_in_region_to_scan as *const std::ffi::c_void,
                        buffer.as_mut_ptr() as *mut std::ffi::c_void,
                        bytes_to_read,
                        &mut bytes_read_count,
                    )
                };

                if read_success != 0 && bytes_read_count > 0 {
                    let actual_buffer = &buffer[..bytes_read_count];
                    for (i, window) in actual_buffer.windows(pattern.len()).enumerate() {
                        if window == pattern {
                            found_addresses.push(address_in_region_to_scan + i);
                            if found_addresses.len() >= max_occurrences {
                                break;
                            }
                        }
                    }
                }
                address_in_region_to_scan += bytes_read_count;
                if bytes_read_count == 0 { // If ReadProcessMemory reads 0 bytes, move to next region
                    break;
                }
            }
        }
        current_address = (mem_info.BaseAddress as usize) + mem_info.RegionSize;
        // Check for overflow if RegionSize is huge
        if current_address < mem_info.BaseAddress as usize {
            break;
        }
    }

    unsafe { CloseHandle(process_handle) };
    Ok(found_addresses)
}

/// Reads a REG_SZ (string) value from the Windows Registry.
pub fn read_registry_sz_value(
    hkey_root: windows_sys::Win32::System::Registry::HKEY, // e.g., HKEY_CURRENT_USER
    sub_key_path: &str,
    value_name: &str,
) -> Result<String> {
    let mut hkey: windows_sys::Win32::System::Registry::HKEY = std::ptr::null_mut();
    let wide_sub_key_path: Vec<u16> = sub_key_path.encode_utf16().chain(std::iter::once(0)).collect();
    let wide_value_name: Vec<u16> = value_name.encode_utf16().chain(std::iter::once(0)).collect();

    let status_open = unsafe {
        windows_sys::Win32::System::Registry::RegOpenKeyExW(
            hkey_root,
            wide_sub_key_path.as_ptr(),
            0, // ulOptions
            windows_sys::Win32::System::Registry::KEY_READ,
            &mut hkey,
        )
    };

    if status_open != 0 { // ERROR_SUCCESS is 0. LSTATUS is i32.
        return Err(anyhow!(
            "Failed to open registry key '{}'. Error code: {}",
            sub_key_path, status_open
        ));
    }

    let mut data_type: u32 = 0;
    let mut buffer_size: u32 = 0; // Size in bytes

    // First call to get the size of the data
    let status_query_size = unsafe {
        windows_sys::Win32::System::Registry::RegQueryValueExW(
            hkey,
            wide_value_name.as_ptr(),
            std::ptr::null_mut(), // lpReserved
            &mut data_type,
            std::ptr::null_mut(), // lpData
            &mut buffer_size,     // lpcbData
        )
    };

    if status_query_size != 0 { // ERROR_SUCCESS is 0
        unsafe { windows_sys::Win32::System::Registry::RegCloseKey(hkey) };
        return Err(anyhow!(
            "Failed to query size of registry value '{}' in key '{}'. Error code: {}",
            value_name, sub_key_path, status_query_size
        ));
    }

    if data_type != windows_sys::Win32::System::Registry::REG_SZ {
        unsafe { windows_sys::Win32::System::Registry::RegCloseKey(hkey) };
        return Err(anyhow!(
            "Registry value '{}' in key '{}' is not REG_SZ type (type: {}).",
            value_name, sub_key_path, data_type
        ));
    }

    if buffer_size == 0 { // Empty string
        unsafe { windows_sys::Win32::System::Registry::RegCloseKey(hkey) };
        return Ok(String::new());
    }
    
    // buffer_size is in bytes. For REG_SZ, it includes the null terminator.
    // Vec<u16> needs number of u16 elements.
    let mut value_buffer: Vec<u16> = vec![0u16; (buffer_size / 2) as usize];
    let mut actual_buffer_size = buffer_size; // Pass the size in bytes

    let status_query_value = unsafe {
        windows_sys::Win32::System::Registry::RegQueryValueExW(
            hkey,
            wide_value_name.as_ptr(),
            std::ptr::null_mut(),
            &mut data_type, // Can be null if type is already known and checked
            value_buffer.as_mut_ptr() as *mut u8,
            &mut actual_buffer_size,
        )
    };

    unsafe { windows_sys::Win32::System::Registry::RegCloseKey(hkey) };

    if status_query_value != 0 { // ERROR_SUCCESS is 0
        return Err(anyhow!(
            "Failed to query value of registry key '{}' value '{}'. Error code: {}",
            sub_key_path, value_name, status_query_value
        ));
    }
    
    // actual_buffer_size will be the size in bytes, including null terminator.
    // Convert to number of u16s, excluding the null terminator for String::from_utf16_lossy
    let num_u16s = (actual_buffer_size / 2) as usize;
    let end_idx = if num_u16s > 0 && value_buffer[num_u16s - 1] == 0 {
        num_u16s - 1 // Exclude null terminator
    } else {
        num_u16s
    };

    Ok(String::from_utf16_lossy(&value_buffer[..end_idx]))
}