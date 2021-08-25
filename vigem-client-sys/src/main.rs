use vigem_client_sys::vigem_alloc;

fn main() {
    println!("{:p}", unsafe { vigem_alloc() });
}
