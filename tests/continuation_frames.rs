include!("../src/continuation-frames.rs.txt");

#[test]
fn aligned_frames_survive_growth_shrink_and_release() {
    unsafe {
        let mut pointer=ink_native_frames_resize(std::ptr::null_mut(),0,64,64);
        assert!(!pointer.is_null());assert_eq!(pointer as usize % 64,0);
        for i in 0..64 {pointer.add(i).write(i as u8);}
        pointer=ink_native_frames_resize(pointer,64,4096,64);
        assert!(!pointer.is_null());assert_eq!(pointer as usize % 64,0);
        for i in 0..64 {assert_eq!(pointer.add(i).read(),i as u8);}
        pointer=ink_native_frames_resize(pointer,4096,128,64);
        assert!(!pointer.is_null());
        for i in 0..64 {assert_eq!(pointer.add(i).read(),i as u8);}
        assert!(ink_native_frames_resize(pointer,128,0,64).is_null());
    }
}

#[test]
fn invalid_new_layouts_leave_existing_frames_owned_by_the_caller() {
    unsafe {
        for (size,alignment) in [(64_000_001,8),(32,0),(32,3),(32,8192)] {
            assert!(ink_native_frames_resize(std::ptr::null_mut(),0,size,alignment).is_null());
        }
        let pointer=ink_native_frames_resize(std::ptr::null_mut(),0,16,8);
        assert!(!pointer.is_null());pointer.write(73);
        assert!(ink_native_frames_resize(pointer,16,64_000_001,8).is_null());
        assert_eq!(pointer.read(),73);
        assert!(ink_native_frames_resize(pointer,16,0,8).is_null());
    }
}
