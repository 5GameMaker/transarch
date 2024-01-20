use transarch::wasm;

extern crate transarch;

#[test]
fn two_are_not_the_same() {
    let a: &[u8] = wasm! {
        #[no_mangle]
        pub extern "C" fn hi() {
            println!("hello 1");
        }
    };
    let b: &[u8] = wasm! {
        #[no_mangle]
        pub extern "C" fn hi() {
            println!("hello 2");
        }
    };

    assert_ne!(a, b);
}
