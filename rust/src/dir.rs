use std::ffi::{c_char, c_int, c_void};
use std::ptr::{self, addr_of, addr_of_mut};
use crate::config::{config_get_string, ConfigStringKey};

#[repr(C)]
pub struct DirListing {
    pub files: *mut *mut c_char,
    pub num_files: c_int,
}

struct DirData {
    listing: DirListing,
    max_files: c_int,
    cased_filename: *mut c_char,
}

static mut DATA: DirData = DirData {
    listing: DirListing {
        files: ptr::null_mut(),
        num_files: 0,
    },
    max_files: 0,
    cased_filename: ptr::null_mut(),
};

const BASE_MAX_FILES: c_int = 100;
const FILE_NAME_MAX: usize = 300;

const TYPE_DIR: c_int = 1;
const TYPE_FILE: c_int = 2;

const LIST_CONTINUE: c_int = 1;
const LIST_MATCH: c_int = 2;

const NOT_LOCALIZED: c_int = 0;
const MUST_BE_LOCALIZED: c_int = 2;

unsafe extern "C" {
    fn platform_file_manager_list_directory_contents(
        dir: *const c_char,
        file_type: c_int,
        extension: *const c_char,
        callback: unsafe extern "C" fn(*const c_char) -> c_int,
    ) -> c_int;
    fn platform_file_manager_compare_filename(a: *const c_char, b: *const c_char) -> c_int;
    fn platform_file_manager_should_case_correct_file() -> c_int;
    fn file_open(filename: *const c_char, mode: *const c_char) -> *mut c_void;
    fn file_close(stream: *mut c_void) -> c_int;
    
    // libc functions
    fn strncpy(dest: *mut c_char, src: *const c_char, n: usize) -> *mut c_char;
    fn strchr(s: *const c_char, c: c_int) -> *mut c_char;
    fn strlen(s: *const c_char) -> usize;
    fn qsort(base: *mut c_void, nmemb: usize, size: usize, compar: unsafe extern "C" fn(*const c_void, *const c_void) -> c_int);
}

unsafe fn allocate_listing_files(min: c_int, max: c_int) {
    unsafe {
        for i in min..max {
            let ptr = libc::malloc(FILE_NAME_MAX) as *mut c_char;
            if !ptr.is_null() {
                *ptr = 0;
                *(*addr_of_mut!(DATA.listing.files)).add(i as usize) = ptr;
            }
        }
    }
}

unsafe fn clear_dir_listing() {
    unsafe {
        (*addr_of_mut!(DATA.listing.num_files)) = 0;
        if *addr_of!(DATA.max_files) <= 0 {
            (*addr_of_mut!(DATA.listing.files)) = libc::malloc(BASE_MAX_FILES as usize * std::mem::size_of::<*mut c_char>()) as *mut *mut c_char;
            allocate_listing_files(0, BASE_MAX_FILES);
            (*addr_of_mut!(DATA.max_files)) = BASE_MAX_FILES;
        } else {
            for i in 0..*addr_of!(DATA.max_files) {
                let ptr = *(*addr_of_mut!(DATA.listing.files)).add(i as usize);
                if !ptr.is_null() {
                    *ptr = 0;
                }
            }
        }
    }
}

unsafe fn expand_dir_listing() {
    unsafe {
        let old_max_files = *addr_of!(DATA.max_files);
        let new_max_files = 2 * old_max_files;
        (*addr_of_mut!(DATA.max_files)) = new_max_files;
        (*addr_of_mut!(DATA.listing.files)) = libc::realloc(
            *addr_of!(DATA.listing.files) as *mut c_void,
            new_max_files as usize * std::mem::size_of::<*mut c_char>(),
        ) as *mut *mut c_char;
        allocate_listing_files(old_max_files, new_max_files);
    }
}

unsafe extern "C" fn compare_lower(va: *const c_void, vb: *const c_void) -> c_int {
    unsafe {
        let a = *(va as *const *const c_char);
        let b = *(vb as *const *const c_char);
        platform_file_manager_compare_filename(a, b)
    }
}

unsafe extern "C" fn add_to_listing(filename: *const c_char) -> c_int {
    unsafe {
        if *addr_of!(DATA.listing.num_files) >= *addr_of!(DATA.max_files) {
            expand_dir_listing();
        }
        let ptr = *(*addr_of_mut!(DATA.listing.files)).add(*addr_of!(DATA.listing.num_files) as usize);
        if !ptr.is_null() {
            strncpy(ptr, filename, FILE_NAME_MAX);
            *ptr.add(FILE_NAME_MAX - 1) = 0;
            (*addr_of_mut!(DATA.listing.num_files)) += 1;
        }
        LIST_CONTINUE
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dir_find_files_with_extension(extension: *const c_char) -> *const DirListing {
    unsafe {
        clear_dir_listing();
        platform_file_manager_list_directory_contents(ptr::null(), TYPE_FILE, extension, add_to_listing);
        if !(*addr_of!(DATA.listing.files)).is_null() {
            qsort(
                *addr_of!(DATA.listing.files) as *mut c_void,
                *addr_of!(DATA.listing.num_files) as usize,
                std::mem::size_of::<*mut c_char>(),
                compare_lower,
            );
        }
        addr_of!(DATA.listing)
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dir_find_all_subdirectories() -> *const DirListing {
    unsafe {
        clear_dir_listing();
        platform_file_manager_list_directory_contents(ptr::null(), TYPE_DIR, ptr::null(), add_to_listing);
        if !(*addr_of!(DATA.listing.files)).is_null() {
            qsort(
                *addr_of!(DATA.listing.files) as *mut c_void,
                *addr_of!(DATA.listing.num_files) as usize,
                std::mem::size_of::<*mut c_char>(),
                compare_lower,
            );
        }
        addr_of!(DATA.listing)
    }
}

unsafe extern "C" fn compare_case(filename: *const c_char) -> c_int {
    unsafe {
        if platform_file_manager_compare_filename(filename, *addr_of!(DATA.cased_filename)) == 0 {
            libc::strcpy(*addr_of_mut!(DATA.cased_filename), filename);
            return LIST_MATCH;
        }
        LIST_CONTINUE
    }
}

unsafe fn correct_case(dir: *const c_char, filename: *mut c_char, file_type: c_int) -> bool {
    unsafe {
        (*addr_of_mut!(DATA.cased_filename)) = filename;
        platform_file_manager_list_directory_contents(dir, file_type, ptr::null(), compare_case) == LIST_MATCH
    }
}

unsafe fn move_left(mut str_ptr: *mut c_char) {
    unsafe {
        while *str_ptr != 0 {
            *str_ptr = *str_ptr.add(1);
            str_ptr = str_ptr.add(1);
        }
    }
}

unsafe fn get_case_corrected_file(dir: *const c_char, filepath: *const c_char) -> *const c_char {
    static mut CORRECTED_FILENAME: [c_char; 2 * FILE_NAME_MAX] = [0; 2 * FILE_NAME_MAX];
    unsafe {
        for i in 0..(2 * FILE_NAME_MAX) { CORRECTED_FILENAME[i] = 0; }
        let mut dir_len = 0;
        let mut actual_dir = dir;
        if !dir.is_null() {
            dir_len = strlen(dir) + 1;
            strncpy(addr_of_mut!(CORRECTED_FILENAME) as *mut c_char, dir, 2 * FILE_NAME_MAX - 1);
            CORRECTED_FILENAME[dir_len - 1] = b'/' as c_char;
        } else {
            actual_dir = ".\0".as_ptr() as *const c_char;
        }

        strncpy((addr_of_mut!(CORRECTED_FILENAME) as *mut c_char).add(dir_len), filepath, 2 * FILE_NAME_MAX - dir_len - 1);

        let fp = file_open(addr_of!(CORRECTED_FILENAME) as *const c_char, "rb\0".as_ptr() as *const c_char);
        if !fp.is_null() {
            file_close(fp);
            return addr_of!(CORRECTED_FILENAME) as *const c_char;
        }

        if platform_file_manager_should_case_correct_file() == 0 {
            return ptr::null();
        }

        let mut slash = strchr((addr_of!(CORRECTED_FILENAME) as *const c_char).add(dir_len), b'/' as c_int);
        if slash.is_null() {
            slash = strchr((addr_of!(CORRECTED_FILENAME) as *const c_char).add(dir_len), b'\\' as c_int);
        }

        if !slash.is_null() {
            *slash = 0;
            if correct_case(actual_dir, (addr_of_mut!(CORRECTED_FILENAME) as *mut c_char).add(dir_len), TYPE_DIR) {
                let path = slash.add(1);
                if *path == b'\\' as c_char {
                    move_left(path);
                }
                if correct_case(addr_of!(CORRECTED_FILENAME) as *const c_char, path, TYPE_FILE) {
                    *slash = b'/' as c_char;
                    return addr_of!(CORRECTED_FILENAME) as *const c_char;
                }
            }
        } else {
            if correct_case(actual_dir, (addr_of_mut!(CORRECTED_FILENAME) as *mut c_char).add(dir_len), TYPE_FILE) {
                return addr_of!(CORRECTED_FILENAME) as *const c_char;
            }
        }
        ptr::null()
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dir_get_file(filepath: *const c_char, localizable: c_int) -> *const c_char {
    unsafe {
        if localizable != NOT_LOCALIZED {
            let custom_dir = config_get_string(ConfigStringKey::UiLanguageDir);
            if !custom_dir.is_null() && *custom_dir != 0 {
                let path = get_case_corrected_file(custom_dir, filepath);
                if !path.is_null() {
                    return path;
                } else if localizable == MUST_BE_LOCALIZED {
                    return ptr::null();
                }
            }
        }
        get_case_corrected_file(ptr::null(), filepath)
    }
}
